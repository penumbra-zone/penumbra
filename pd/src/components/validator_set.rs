use std::{
    borrow::{Borrow, BorrowMut},
    collections::BTreeMap,
};

use anyhow::Result;

use futures::Future;
use penumbra_crypto::{
    asset::{self, Denom, Id},
    Address,
};
use penumbra_proto::Protobuf;
use sqlx::{query, Postgres, Transaction};
use tendermint::{
    abci::types::{Evidence, ValidatorUpdate},
    PublicKey,
};

use crate::state::Reader;
use penumbra_stake::{
    BaseRateData, Epoch, FundingStream, IdentityKey, RateData, Validator, ValidatorDefinition,
    ValidatorInfo, ValidatorState, ValidatorStateName, ValidatorStatus,
    VerifiedValidatorDefinition, STAKING_TOKEN_ASSET_ID, STAKING_TOKEN_DENOM,
};

#[derive(Debug, Clone)]
struct Cache {
    /// Records complete validator states as they change during the course of the block.
    ///
    /// Updated as changes occur during the block, but will not be persisted
    /// to the database until the block is committed.
    validator_set: BTreeMap<IdentityKey, ValidatorInfo>,
    /// Indicates the current epoch.
    epoch: Epoch,
}

impl Cache {
    async fn load(reader: &Reader, epoch: Epoch) -> Result<Self> {
        let mut validator_set = BTreeMap::new();
        for validator in reader.validator_info(true).await? {
            validator_set.insert(validator.validator.identity_key.clone(), validator.clone());
        }

        Ok(Self {
            validator_set,
            epoch,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockChanges {
    /// New validators added during the block. Saved and available for staking when the block is committed.
    pub new_validators: BTreeMap<IdentityKey, Vec<VerifiedValidatorDefinition>>,
    /// Existing validators updated during the block. Saved when the block is committed, queued for the next epoch.
    pub pending_validator_updates: Vec<VerifiedValidatorDefinition>,
    /// Validators slashed during this block. Saved when the block is committed.
    ///
    /// The validator's rate will have a slashing penalty immediately applied during the current epoch.
    /// Their future rates will be held constant.
    pub slashed_validators: Vec<IdentityKey>,
    /// The net delegations performed in this block per validator.
    pub delegation_changes: BTreeMap<IdentityKey, i64>,
    /// The list of updated validator identity keys and powers to send to Tendermint during `end_block`.
    pub tm_validator_updates: Vec<ValidatorUpdate>,
    /// If this is the last block in an epoch, epoch-related changes are recorded here.
    pub epoch_changes: Option<EpochChanges>,
}

impl BlockChanges {
    pub async fn commit(self, dbtx: &mut Transaction<'_, Postgres>) -> Result<()> {
        tracing::debug!(?height, "Committing block");
        tracing::debug!("end height {}", self.epoch().end_height().value(),);

        // Track the net change in delegations in this block.
        let epoch_index = self.epoch().index;
        for (identity_key, delegation_change) in &self.delegation_changes {
            query!(
                "INSERT INTO delegation_changes VALUES ($1, $2, $3)",
                identity_key.encode_to_vec(),
                epoch_index as i64,
                delegation_change
            )
            .execute(&mut *dbtx)
            .await?;
        }

        // Queue redefinitions of existing validators to be processed at the epoch boundary
        for v in &self.updated_validators {
            query!(
                "INSERT INTO pending_validator_updates (definition) VALUES ($1)",
                v.encode_to_vec()
            )
            .execute(&mut *dbtx)
            .await?;
        }

        // Handle adding newly added validators with default rates
        for (ik, defs) in &self.new_validators {
            // Sort the validator definitions by sequence number + tiebreaker,
            // in case there are conflicts.
            let mut defs = defs.clone();
            defs.sort();
            let v = defs.pop().expect("new_validators has no empty lists");

            query!(
                "INSERT INTO validators (
                    identity_key,
                    consensus_key,
                    sequence_number,
                    name,
                    website,
                    description,
                    voting_power,
                    validator_state,
                    unbonding_epoch
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                v.validator.identity_key.encode_to_vec(),
                v.validator.consensus_key.to_bytes(),
                i64::try_from(v.validator.sequence_number)?,
                v.validator.name,
                v.validator.website,
                v.validator.description,
                // New validators have initial voting power of 0
                0,
                // New validators start in Inactive state
                ValidatorStateName::Inactive.to_str().to_string(),
                Option::<i64>::None,
            )
            .execute(&mut *dbtx)
            .await?;

            for FundingStream { address, rate_bps } in v.validator.funding_streams.as_ref() {
                query!(
                    "INSERT INTO validator_fundingstreams (
                        identity_key,
                        address,
                        rate_bps
                    ) VALUES ($1, $2, $3)",
                    v.validator.identity_key.encode_to_vec(),
                    address.to_string(),
                    *rate_bps as i32,
                )
                .execute(&mut *dbtx)
                .await?;
            }

            // Delegations require knowing the rates for the
            // next epoch, so pre-populate with 0 reward => exchange rate 1 for
            // the current and next epochs.
            for epoch in [epoch_index, epoch_index + 1] {
                query!(
                    "INSERT INTO validator_rates (
                    identity_key,
                    epoch,
                    validator_reward_rate,
                    validator_exchange_rate
                ) VALUES ($1, $2, $3, $4)",
                    v.validator.identity_key.encode_to_vec(),
                    epoch as i64,
                    0,
                    1_0000_0000i64, // 1 represented as 1e8
                )
                .execute(&mut *dbtx)
                .await?;
            }
        }

        // Slash validators by updating their state to "Slashed" and
        // writing the slashed exchange rate.
        for ik in &self.slashed_validators {
            // The exchange rate was already updated during `slash_validator`,
            // so we just need to rewrite the rate for the current epoch.
            let slashed_rate = self
                .cache
                .validator_set
                .get(&ik)
                .expect("slashed validator must be known")
                .rate_data;
            query!(
                    "UPDATE validator_rates SET validator_exchange_rate=$1 WHERE identity_key=$2 AND epoch=$3",
                    slashed_rate.validator_exchange_rate as i64,
                    slashed_rate.identity_key.encode_to_vec(),
                    slashed_rate.epoch_index as i64,
            )
            .execute(&mut *dbtx)
            .await?;

            // Next, update the validator state to reflect that the validator
            // was slashed, has no voting power, and is no longer unbonding (if
            // it was at the time of slashing).
            query!(
                "UPDATE validators SET validator_state=$2, voting_power=$3, unbonding_epoch=$4 WHERE identity_key = $1",
                ik.borrow().encode_to_vec(),
                ValidatorStateName::Slashed.to_str(),
                0,
                None,
            )
            .execute(&mut *dbtx)
            .await?;
        }

        // TODO: should this be part of a future shielded pool component?
        // Save any new assets found in the block to the asset registry.
        for (id, asset) in &self.supply_updates {
            query!(
                "INSERT INTO assets (asset_id, denom, total_supply)
            VALUES ($1, $2, $3)
            ON CONFLICT (asset_id) DO UPDATE SET denom=$2, total_supply=$3",
                &id.to_bytes()[..],
                asset.0.to_string(),
                asset.1 as i64
            )
            .execute(&mut *dbtx)
            .await?;
        }

        if let Some(epoch_changes) = self.epoch_changes.take() {
            epoch_changes.commit(dbtx).await?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EpochChanges {
    /// Base rates for the next epoch go here.
    pub next_base_rate: BaseRateData,
    /// Validator rates for the next epoch go here.
    pub next_rates: Vec<RateData>,
    /// Set in `end_epoch` and reset to `None` when `reward_notes` is called.
    // TODO: produce in end_epoch and return to caller, make this private
    pub reward_notes: Vec<(u64, Address)>,
    /// Updates to existing validator definitions that are applied as part of
    /// this epoch transition.
    pub updated_validators: Vec<ValidatorDefinition>,
    /// Records updates to the supply of staking or delegation tokens as a result of an epoch transition.
    pub supply_updates: BTreeMap<asset::Id, (asset::Denom, u64)>,
}

impl EpochChanges {
    pub async fn commit(self, dbtx: &mut Transaction<'_, Postgres>) -> Result<()> {
        tracing::debug!("commit epoch");
        tracing::debug!(?self.next_base_rate, "Saving next base rate to the database");
        query!(
            "INSERT INTO base_rates VALUES ($1, $2, $3)",
            self.next_base_rate.epoch_index as i64,
            self.next_base_rate.base_reward_rate as i64,
            self.next_base_rate.base_exchange_rate as i64,
        )
        .execute(&mut *dbtx)
        .await?;

        for rate in self.next_rates {
            query!(
                    // This query needs to be ON CONFLICT UPDATE because this rate will have already been set
                    // in the case of a new validator.
                    "INSERT INTO validator_rates VALUES ($1, $2, $3, $4) ON CONFLICT ON CONSTRAINT validator_rates_pkey
                    DO UPDATE SET validator_reward_rate=$3, validator_exchange_rate=$4",
                    rate.identity_key.encode_to_vec(),
                    rate.epoch_index as i64,
                    rate.validator_reward_rate as i64,
                    rate.validator_exchange_rate as i64,
                )
                .execute(&mut *dbtx)
                .await?;
        }

        // TODO: write the updated validators
        // TODO: write the supply updates
        // TODO: what processes the reward notes?
        // -- problem: how do we connect this code with the code that handles the shielded pool?

        Ok(())
    }
}

/// Records the complete state of all validators throughout a block,
/// and is responsible for producing the necessary database queries
/// for persistence.
///
/// After calling `ValidatorSet.commit`, the block is advanced.
/// Internal tracking of validator changes is reset for the new block.
#[derive(Debug, Clone)]
pub struct ValidatorSet {
    /// Changes to the validator set during the course of the block.
    /// Will be reset every time `end_block` is called.
    block_changes: Option<BlockChanges>,
    /// Cache of validator states.
    ///
    /// This should only be written to in end_block, immediately before commit.
    cache: Cache,
    /// Database reader.
    reader: Reader,
}

impl ValidatorSet {
    pub async fn new(reader: Reader, epoch: Epoch) -> Result<Self> {
        let cache = Cache::load(&reader, epoch).await?;

        Ok(ValidatorSet {
            cache,
            reader,
            block_changes: None,
        })
    }

    // TODO: maybe the begin/end/commit flow should be a trait or something
    pub async fn begin_block(&mut self) -> Result<()> {
        self.cache = Cache::load(&self.reader).await?;
        self.block_changes = Some(Default::default());

        Ok(())
    }

    // Called during `end_block`. Responsible for resolving conflicting ValidatorDefinitions
    // that came in during the block and updating `validator_set` with new validator info.
    //
    // Any *state changes* (i.e. ValidatorState) should have already been applied to `validator_set`
    // by the time this is called!
    pub async fn end_block(&mut self, height: u64) -> Result<()> {
        let epoch = self.epoch().clone();

        if epoch.end_height().value() == height {
            self.end_epoch().await?;
        }

        // TODO: update the below code to reflect that we'll build the list of
        // validator changes differently

        // Set `self.tm_validator_updates` to the complete set of
        // validators and voting power. This must be the last step performed,
        // after all voting power calculations and validator state transitions have
        // been completed.
        //
        // TODO: It could be more efficient to only return the power of
        // updated validators.
        self.block_changes
            .as_mut()
            .expect("block_changes should be initialized during begin_block")
            .tm_validator_updates = self
            .validators_info()
            .map(|v| {
                let v = v.borrow();
                // if the validator is non-Active, set their voting power as
                // returned to Tendermint to 0. Only Active validators report
                // voting power to Tendermint.
                tracing::debug!(?v, "calculating validator power in end_block");
                let power = if v.status.state == ValidatorState::Active {
                    v.status.voting_power as u64
                } else {
                    0
                };
                let validator = &v.validator;
                let pub_key = validator.consensus_key;
                Ok(tendermint::abci::types::ValidatorUpdate {
                    pub_key,
                    power: power.try_into()?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(())
    }

    pub async fn commit(&mut self, dbtx: &mut Transaction<'_, Postgres>) -> Result<()> {
        self.block_changes.take().unwrap().commit(dbtx).await
    }

    pub fn update_delegations(&mut self, delegation_changes: &BTreeMap<IdentityKey, i64>) {
        // Tally the delegation changes in this transaction
        for (identity_key, delegation_change) in delegation_changes {
            *self
                .block_changes
                .as_mut()
                .expect("block_changes should be initialized during begin_block")
                .delegation_changes
                .entry(identity_key.clone())
                .or_insert(0) += delegation_change;
        }
    }

    /// Called during the commit phase of a block. Will return the current status of
    /// all validators within the state machine.
    pub fn next_validator_statuses(&self) -> Vec<ValidatorStatus> {
        self.validators_info()
            .map(|v| v.borrow().status.clone())
            .collect()
    }

    /// Called during `end_epoch`. Will perform state transitions to validators based
    /// on changes to voting power that occurred in this epoch.
    pub fn process_epoch_transitions(
        &mut self,
        active_validator_limit: u64,
        unbonding_epochs: u64,
    ) -> Result<()> {
        // Sort the next validator states by voting power.
        // Dislike this clone, but the borrow checker was complaining about the loop modifying itself
        // when I tried using the validators_info() iterator.
        let mut validators_info = self
            .cache
            .validator_set
            .iter()
            .map(|(_, v)| (v.clone()))
            .collect::<Vec<_>>();
        validators_info.sort_by(|a, b| {
            a.borrow()
                .status
                .voting_power
                .cmp(&b.borrow().status.voting_power)
        });
        let top_validators = validators_info
            .iter()
            .take(active_validator_limit as usize)
            .map(|v| v.borrow().validator.identity_key.clone())
            .collect::<Vec<_>>();
        for vi in &validators_info {
            let validator_status = &vi.borrow().status.clone();
            if validator_status.state == ValidatorState::Inactive
                || matches!(
                    validator_status.state,
                    ValidatorState::Unbonding { unbonding_epoch: _ }
                )
            {
                // If an Inactive or Unbonding validator is in the top `active_validator_limit` based
                // on voting power and the delegation pool has a nonzero balance,
                // then the validator should be moved to the Active state.
                if top_validators.contains(&validator_status.identity_key) {
                    // TODO: How do we check the delegation pool balance here?
                    // https://github.com/penumbra-zone/penumbra/issues/445
                    self.activate_validator(vi.borrow().validator.consensus_key.clone())?;
                }
            } else if validator_status.state == ValidatorState::Active {
                // An Active validator could also be displaced and move to the
                // Unbonding state.
                if !top_validators.contains(&validator_status.identity_key) {
                    self.unbond_validator(
                        vi.borrow().validator.consensus_key.clone(),
                        self.epoch().index + unbonding_epochs,
                    )?;
                }
            }

            // An Unbonding validator can become Inactive if the unbonding period expires
            // and the validator is still in Unbonding state
            if let ValidatorState::Unbonding { unbonding_epoch } = validator_status.state {
                if unbonding_epoch <= self.epoch().index {
                    self.deactivate_validator(vi.borrow().validator.consensus_key.clone())?;
                }
            };
        }

        Ok(())
    }

    /// Called during `end_epoch`. Will calculate validator changes that can only happen during epoch changes
    /// such as rate updates.
    async fn end_epoch(&mut self) -> Result<()> {
        // We've finished processing the last block of `epoch`, so we've
        // crossed the epoch boundary, and (prev | current | next) are:
        let prev_epoch = self.epoch();
        let current_epoch = prev_epoch.next();
        let next_epoch = current_epoch.next();

        tracing::info!(
            ?prev_epoch,
            ?current_epoch,
            ?next_epoch,
            "crossed epoch boundary, processing rate updates"
        );
        metrics::increment_counter!("epoch");

        let unbonding_epochs = self.reader.chain_params_rx().borrow().unbonding_epochs;
        let active_validator_limit = self
            .reader
            .chain_params_rx()
            .borrow()
            .active_validator_limit;

        // Pull out pending validator redefinitions and apply them to the cache,
        // then queue them to be written to the state.
        let mut updated_validators = Vec::new();
        for (ik, mut defs) in self.reader.pending_validator_redefinitions().await? {
            // Select the validator redefinition with the highest sequence number.
            defs.sort();
            let update = defs.pop().expect("redefinitions list is nonempty");
            tracing::info!(?update, "processing queued redefinition");
            // Overwrite the cache with the new update, so we'll use it while computing new
            // rates.  Because we're in EndBlock, we know the "divergent" data will be reconciled
            // when we write the updates in Commit and reload the cache.
            self.cache
                .validator_set
                .get_mut(&ik)
                .expect("redefined validators must exist")
                .validator = update.validator.clone();
            updated_validators.push(update);
        }

        tracing::debug!("processing base rate");
        let current_base_rate = self.reader.base_rate_data(current_epoch.index).await?;

        // We are calculating the rates for the next epoch. For example, if
        // we have just ended epoch 2 and are entering epoch 3, we are calculating the rates
        // for epoch 4.

        /// FIXME: set this less arbitrarily, and allow this to be set per-epoch
        /// 3bps -> 11% return over 365 epochs, why not
        const BASE_REWARD_RATE: u64 = 3_0000;

        let next_base_rate = current_base_rate.next(BASE_REWARD_RATE);

        // rename to curr_rate so it lines up with next_rate (same # chars)
        tracing::debug!(curr_base_rate = ?current_base_rate);
        tracing::debug!(?next_base_rate);

        let mut staking_token_supply = self
            .reader
            .asset_lookup(*STAKING_TOKEN_ASSET_ID)
            .await?
            .map(|info| info.total_supply)
            .unwrap();

        let mut next_rates = Vec::new();
        let mut reward_notes = Vec::new();
        let mut supply_updates = BTreeMap::new();

        // this is a bit complicated: because we're in the EndBlock phase, and the
        // delegations in this block have not yet been committed, we have to combine
        // the delegations in pending_block with the ones already committed to the
        // state. otherwise the delegations committed in the epoch threshold block
        // would be lost.
        //
        // TODO: encapsulate the delegations logic
        let mut delegation_changes = self.reader.delegation_changes(prev_epoch.index).await?;
        for (id_key, delta) in &self
            .block_changes
            .as_ref()
            .expect("block_changes should be initialized during begin_block")
            .delegation_changes
        {
            // TODO: does this need to be copied back to `self.block_changes.delegation_changes`
            // at the end of this method, so that `commit_block` will be able to use any
            // changes?
            *delegation_changes.entry(id_key.clone()).or_insert(0) += delta;
        }

        // steps (foreach validator):
        // - get the total token supply for the validator's delegation tokens
        // - process the updates to the token supply:
        //   - collect all delegations occurring in previous epoch and apply them (adds to supply);
        //   - collect all undelegations started in previous epoch and apply them (reduces supply);
        // - feed the updated (current) token supply into current_rates.voting_power()
        // - persist both the current voting power and the current supply
        //
        for v in &mut self.cache.validator_set {
            let validator = v.1;
            let current_rate = validator.rate_data.clone();
            tracing::debug!(?validator, "processing validator rate updates");
            assert!(current_rate.epoch_index == current_epoch.index);

            let funding_streams = self
                .reader
                .funding_streams(validator.validator.identity_key.clone())
                .await?;

            let next_rate = current_rate.next(
                &next_base_rate,
                funding_streams.as_ref(),
                &validator.status.state,
            );
            assert!(next_rate.epoch_index == current_epoch.index + 1);
            let identity_key = validator.validator.identity_key.clone();

            let delegation_delta = delegation_changes.get(&identity_key).unwrap_or(&0i64);

            let delegation_amount = delegation_delta.abs() as u64;
            let unbonded_amount = current_rate.unbonded_amount(delegation_amount);

            let mut delegation_token_supply = self
                .reader
                .asset_lookup(identity_key.delegation_token().id())
                .await?
                .map(|info| info.total_supply)
                .unwrap_or(0);

            if *delegation_delta > 0 {
                // net delegation: subtract the unbonded amount from the staking token supply
                staking_token_supply = staking_token_supply.checked_sub(unbonded_amount).unwrap();
                delegation_token_supply = delegation_token_supply
                    .checked_add(delegation_amount)
                    .unwrap();
            } else {
                // net undelegation: add the unbonded amount to the staking token supply
                staking_token_supply = staking_token_supply.checked_add(unbonded_amount).unwrap();
                delegation_token_supply = delegation_token_supply
                    .checked_sub(delegation_amount)
                    .unwrap();
            }

            // update the delegation token supply
            supply_updates.insert(
                identity_key.delegation_token().id(),
                (
                    identity_key.delegation_token().denom(),
                    delegation_token_supply,
                ),
            );
            let voting_power =
                current_rate.voting_power(delegation_token_supply, &current_base_rate);
            tracing::debug!(?voting_power);

            // Update the status of the validator within the validator set
            // with the newly starting epoch's calculated voting rate and power.
            validator.rate_data = current_rate.clone();
            validator.status.voting_power = voting_power;

            // Only Active validators produce commission rewards
            if validator.status.state == ValidatorState::Active {
                // distribute validator commission
                for stream in funding_streams {
                    let commission_reward_amount = stream.reward_amount(
                        delegation_token_supply,
                        &next_base_rate,
                        &current_base_rate,
                    );

                    reward_notes.push((commission_reward_amount, stream.address));
                }
            }

            // rename to curr_rate so it lines up with next_rate (same # chars)
            let delegation_denom = identity_key.delegation_token().denom();
            tracing::debug!(curr_rate = ?current_rate);
            tracing::debug!(?next_rate);
            tracing::debug!(?delegation_delta);
            tracing::debug!(?delegation_token_supply);
            tracing::debug!(?delegation_denom);

            next_rates.push(next_rate);
        }

        tracing::debug!(?staking_token_supply);
        supply_updates.insert(
            *STAKING_TOKEN_ASSET_ID,
            (STAKING_TOKEN_DENOM.clone(), staking_token_supply),
        );

        // State transitions on epoch change are handled here
        // after all rates have been calculated
        self.process_epoch_transitions(active_validator_limit, unbonding_epochs)?;

        self.block_changes.as_mut().unwrap().epoch_changes = Some(EpochChanges {
            next_rates,
            next_base_rate,
            reward_notes,
            supply_updates,
            updated_validators,
        });

        Ok(())
    }

    // This should *only* be called during `end_block` as validators don't
    // get exposed to consensus until then.
    pub fn add_validator(&mut self, validator: ValidatorInfo) {
        self.cache
            .validator_set
            .insert(validator.validator.identity_key.clone(), validator.clone());

        // Add the validator to the block's new validators list so an INSERT query will be generated in `commit_block`.
        self.block_changes
            .as_mut()
            .expect("block_changes should be initialized during begin_block")
            .new_validators
            .insert(validator.validator.identity_key.clone(), validator);
    }

    /// This keeps track of validator definitions received during the block.
    /// Any conflicts will be resolved and resulting changes to the validator set
    /// will be applied during end_block.
    ///
    /// Validator definitions must have been already validated by verify_stateful/verify_stateless
    /// prior to calling this method.
    pub fn add_validator_definition(&mut self, validator_definition: VerifiedValidatorDefinition) {
        let identity_key = validator_definition.validator.identity_key.clone();

        if self
            .validators()
            .any(|v| v.borrow().identity_key == identity_key)
        {
            // If this is an existing validator, we may be changing the rates, because the funding streams may have changed,
            // so queue the redefinition to be processed at the next epoch boundary.
            self.block_changes
                .as_mut()
                .expect("block_changes should be initialized during begin_block")
                .pending_validator_updates
                .push(validator_definition);
        } else {
            // Add the validator to the block's new validators list so an INSERT query will be generated in `commit_block`.
            self.block_changes
                .as_mut()
                .expect("block_changes should be initialized during begin_block")
                .new_validators
                .entry(identity_key)
                .or_default()
                .push(validator_definition);
        }
    }

    pub fn get_validator_info(&self, identity_key: &IdentityKey) -> Option<&ValidatorInfo> {
        self.cache.validator_set.get(identity_key)
    }

    pub fn get_state(&self, identity_key: &IdentityKey) -> Option<&ValidatorState> {
        self.cache
            .validator_set
            .get(identity_key)
            .map(|x| &x.status.state)
    }

    pub fn validators(&self) -> impl Iterator<Item = impl Borrow<&'_ Validator>> {
        self.cache.validator_set.iter().map(|v| &v.1.validator)
    }

    pub fn validators_info(
        &self,
    ) -> impl Clone + Iterator<Item = impl Borrow<&'_ ValidatorInfo> + BorrowMut<&'_ ValidatorInfo>>
    {
        self.cache.validator_set.iter().map(|v| v.1)
    }

    /// Returns all validators that are currently in the `Slashed` state.
    pub fn slashed_validators(&self) -> impl Iterator<Item = impl Borrow<&'_ Validator>> {
        self.cache
            .validator_set
            .iter()
            .filter(|v| v.1.status.state == ValidatorState::Slashed)
            .map(|v| &v.1.validator)
    }

    pub fn unslashed_validators(&self) -> impl Iterator<Item = impl Borrow<&'_ Validator>> {
        // validators: Option<impl IntoIterator<Item = impl Borrow<&'a IdentityKey>>>,
        self.cache
            .validator_set
            .iter()
            // Return all validators that are *not slashed*
            .filter(|v| v.1.status.state != ValidatorState::Slashed)
            .map(|v| &v.1.validator)
    }

    /// Mark a validator as deactivated. Only validators in the unbonding state
    /// can be deactivated.
    pub fn deactivate_validator(&mut self, ck: PublicKey) -> Result<()> {
        tracing::debug!(?ck, "deactivate_validator");
        // Don't love this clone.
        let validator = self.get_validator_by_consensus_key(&ck)?.clone();

        let current_info = self
            .get_validator_info(&validator.identity_key)
            .ok_or(anyhow::anyhow!("Validator not found in state machine"))?;
        let current_state = current_info.status.state;

        match current_state {
            ValidatorState::Unbonding { unbonding_epoch: _ } => {
                self.cache
                    .validator_set
                    .get_mut(&validator.identity_key)
                    .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                    .status
                    .state = ValidatorState::Inactive;
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Validator {} is not in unbonding state",
                validator.identity_key
            )),
        }
    }

    // Activate a validator. Only validators in the inactive or unbonding state
    // may be activated.
    pub fn activate_validator(&mut self, ck: PublicKey) -> Result<()> {
        tracing::debug!(?ck, "activate_validator");
        // Don't love this clone.
        let validator = self.get_validator_by_consensus_key(&ck)?.clone();

        let current_info = self
            .get_validator_info(&validator.identity_key)
            .ok_or(anyhow::anyhow!("Validator not found in state machine"))?;
        let current_state = current_info.status.state;

        let mut mark_active = |validator: &Validator| -> Result<()> {
            self.cache
                .validator_set
                .get_mut(&validator.identity_key)
                .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                .status
                .state = ValidatorState::Active;
            Ok(())
        };

        match current_state {
            ValidatorState::Inactive => mark_active(&validator),
            // The unbonding epoch is not checked here. It is checked in the
            // consensus worker.
            ValidatorState::Unbonding { unbonding_epoch: _ } => mark_active(&validator),
            _ => Err(anyhow::anyhow!(
                "only validators in the inactive or unbonding state may be activated"
            )),
        }
    }

    // Marks a validator as slashed. Only validators in the active or unbonding
    // state may be slashed.
    #[tracing::instrument(skip(self))]
    pub fn slash_validator(&mut self, evidence: &Evidence) -> Result<()> {
        let ck = tendermint::PublicKey::from_raw_ed25519(&evidence.validator.address)
            .ok_or_else(|| anyhow::anyhow!("invalid ed25519 consensus pubkey from tendermint"))
            .unwrap();

        let slashing_penalty = self.reader.chain_params_rx().borrow().slashing_penalty;
        // Don't love this clone.
        let validator = self.get_validator_by_consensus_key(&ck)?.clone();

        tracing::info!(?validator, ?slashing_penalty, "slashing validator");

        let current_info = self
            .get_validator_info(&validator.identity_key)
            .ok_or(anyhow::anyhow!("Validator not found in state machine"))?;
        let current_state = current_info.status.state;

        let mut mark_slashed = |validator: &Validator| -> Result<()> {
            // It's safe to modify the cache here, as it will be reset
            // during begin_block. We need to access the updated state/rate data
            // during end_block and commit.
            let mut cv = self
                .cache
                .validator_set
                .get_mut(&validator.identity_key)
                .ok_or_else(|| anyhow::anyhow!("Validator not found"))?;

            cv.status.state = ValidatorState::Slashed;
            cv.rate_data = cv.rate_data.slash(slashing_penalty);

            self.block_changes
                .as_mut()
                .expect("block_changes should be initialized during begin_block")
                .slashed_validators
                .push(validator.identity_key.clone());

            Ok(())
        };

        match current_state {
            ValidatorState::Active => mark_slashed(&validator),
            ValidatorState::Unbonding { unbonding_epoch: _ } => mark_slashed(&validator),
            _ => Err(anyhow::anyhow!(
                "only validators in the active or unbonding state may be slashed"
            )),
        }
    }

    // Marks a validator as unbonding. Only validators in the active state
    // may begin unbonding.
    pub fn unbond_validator(&mut self, ck: PublicKey, unbonding_epoch: u64) -> Result<()> {
        tracing::debug!(?ck, "unbond_validator");
        // Don't love this clone.
        let validator = self.get_validator_by_consensus_key(&ck)?.clone();
        let current_info = self
            .get_validator_info(&validator.identity_key)
            .ok_or(anyhow::anyhow!("Validator not found in state machine"))?;
        let current_state = current_info.status.state;

        match current_state {
            ValidatorState::Active => {
                // Unbonding the validator means that it can no longer participate
                // in consensus, so its voting power is set to 0.
                self.cache
                    .validator_set
                    .get_mut(&validator.identity_key)
                    .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                    .status
                    .voting_power = 0;
                self.cache
                    .validator_set
                    .get_mut(&validator.identity_key)
                    .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                    .status
                    .state = ValidatorState::Unbonding { unbonding_epoch };
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "only validators in the active state may begin unbonding"
            )),
        }
    }

    // Tendermint validators are referenced to us by their Tendermint consensus key,
    // but we reference them by their Penumbra identity key.
    pub fn get_validator_by_consensus_key(&self, ck: &PublicKey) -> Result<&Validator> {
        let validator = self
            .cache
            .validator_set
            .iter()
            .find(|v| v.1.validator.consensus_key == *ck)
            .ok_or(anyhow::anyhow!("No validator found"))?;
        Ok(&validator.1.validator)
    }

    pub fn epoch(&self) -> Epoch {
        self.cache.epoch
    }
}
