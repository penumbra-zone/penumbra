use std::{
    borrow::{Borrow, BorrowMut},
    collections::BTreeMap,
};

use anyhow::Result;

use penumbra_crypto::{
    asset::{self, Denom, Id},
    Address,
};
use tendermint::{abci::types::ValidatorUpdate, PublicKey};

use crate::state::Reader;
use penumbra_stake::{
    BaseRateData, Epoch, IdentityKey, RateData, Validator, ValidatorDefinition, ValidatorInfo,
    ValidatorState, ValidatorStatus, STAKING_TOKEN_ASSET_ID, STAKING_TOKEN_DENOM,
};

#[derive(Debug, Clone)]
/// Records the complete state of all validators throughout a block,
/// and is responsible for producing the necessary database queries
/// for persistence.
///
/// After calling `ValidatorSet.commit`, the block is advanced.
/// Internal tracking of validator changes is reset for the new block.
pub struct ValidatorSet {
    /// Records complete validator states as they change during the course of the block.
    ///
    /// Updated as changes occur during the block, but will not be persisted
    /// to the database until the block is committed.
    validator_set: BTreeMap<IdentityKey, ValidatorInfo>,
    /// Database reader.
    reader: Reader,
    /// Validator definitions added during the block. Since multiple definitions could
    /// come in for the same validator during a block, we need to deterministically pick
    /// one definition to use.
    ///
    /// If the definition is for an existing validator, this will be pushed to `self.updated_validators`
    /// during end_block.
    ///
    /// Otherwise if the definition is for a new validator, this will be pushed to `self.new_validators`
    /// during end_block.
    validator_definitions: BTreeMap<IdentityKey, Vec<ValidatorDefinition>>,
    /// New validators added during the block. Saved and available for staking when the block is committed.
    pub new_validators: Vec<ValidatorInfo>,
    /// Existing validators updated during the block. Saved when the block is committed.
    pub updated_validators: Vec<ValidatorInfo>,
    /// Validators slashed during this block. Saved when the block is committed.
    ///
    /// The validator's rate will have a slashing penalty immediately applied during the current epoch.
    /// Their future rates will be held constant.
    pub slashed_validators: Vec<IdentityKey>,
    /// The net delegations performed in this block per validator.
    pub delegation_changes: BTreeMap<IdentityKey, i64>,
    /// Indicates the epoch the block belongs to.
    pub epoch: Option<Epoch>,
    /// If this is the last block of an epoch, base rates for the next epoch go here.
    pub next_base_rate: Option<BaseRateData>,
    /// If this is the last block of an epoch, validator rates for the next epoch go here.
    pub next_rates: Option<Vec<RateData>>,
    /// If this is the last block of an epoch, validator statuses for the next epoch go here.
    pub next_validator_statuses: Vec<ValidatorStatus>,
    /// The list of updated validator identity keys and powers to send to Tendermint during `end_block`.
    ///
    /// Set in `end_block` and reset to `None` when `tm_validator_updates` is called.
    pub tm_validator_updates: Vec<ValidatorUpdate>,
    /// Set in `end_epoch` and reset to `None` when `reward_notes` is called.
    pub reward_notes: Vec<(u64, Address)>,
    /// Records any updates to the token supply of some asset that happened in this block.
    pub supply_updates: BTreeMap<asset::Id, (asset::Denom, u64)>,
}

impl ValidatorSet {
    pub async fn new(reader: Reader) -> Result<Self> {
        // Grab all validator info from the database. This will only happen when the
        // ValidatorSet is first instantiated.
        let block_validators = reader.validator_info(true).await?;

        // Initialize all state machine validator states to their current state from the block validators.
        let mut validator_set = BTreeMap::new();
        for validator in block_validators.iter() {
            validator_set.insert(validator.validator.identity_key.clone(), validator.clone());
        }

        Ok(ValidatorSet {
            validator_set,
            epoch: None,
            next_base_rate: None,
            next_rates: None,
            next_validator_statuses: Vec::new(),
            validator_definitions: BTreeMap::new(),
            new_validators: Vec::new(),
            updated_validators: Vec::new(),
            slashed_validators: Vec::new(),
            reader,
            tm_validator_updates: Vec::new(),
            delegation_changes: BTreeMap::new(),
            reward_notes: Vec::new(),
            supply_updates: BTreeMap::new(),
        })
    }

    /// Called during `commit_block` and will reset internal state
    /// for the next block, as well as set `self.epoch` to the
    /// `new_epoch` value passed in.
    pub async fn commit_block(&mut self, new_epoch: Epoch) {
        if new_epoch.index != self.epoch.as_ref().unwrap().index {
            // This resets the rate and supply information
            // that only changes during epoch transitions.
            self.epoch = Some(new_epoch);
            self.next_base_rate = None;
            self.next_rates = None;
            // TODO: next_validator_statuses is a subset of the data
            // within self.updated_validators, this is bad design
            self.next_validator_statuses = Vec::new();
            self.reward_notes = Vec::new();
            self.supply_updates = BTreeMap::new();
        }

        // New, slashed, and updated validators can happen in any block,
        // not just on epoch transitions.
        self.new_validators = Vec::new();
        self.slashed_validators = Vec::new();
        self.validator_definitions = BTreeMap::new();
        self.updated_validators = Vec::new();
        self.tm_validator_updates = Vec::new();
        self.delegation_changes = BTreeMap::new();
    }

    // Called during `end_block`. Responsible for resolving conflicting ValidatorDefinitions
    // that came in during the block and updating `validator_set` with new validator info.
    //
    // Will update the epoch to whatever is passed in.
    //
    // Any *state changes* (i.e. ValidatorState) should have already been applied to `validator_set`
    // by the time this is called!
    pub async fn end_block(&mut self, epoch: Epoch) -> Result<()> {
        self.epoch = Some(epoch.clone());
        // This closure is used to generate a new ValidatorInfo from a ValidatorDefinition.
        // This should *only* be called for *new validators* as it sets the validator's state
        // to Inactive and sets default rate data!
        let make_validator = |v: ValidatorDefinition| -> ValidatorInfo {
            ValidatorInfo {
                validator: v.validator.clone(),
                status: ValidatorStatus {
                    identity_key: v.validator.identity_key.clone(),
                    // Voting power for inactive validators is 0
                    voting_power: 0,
                    state: ValidatorState::Inactive,
                },
                rate_data: RateData {
                    identity_key: v.validator.identity_key,
                    epoch_index: epoch.index,
                    // Validator reward rate is held constant for inactive validators.
                    // Stake committed to inactive validators earns no rewards.
                    validator_reward_rate: 0,
                    // Exchange rate for inactive validators is held constant
                    // and starts at 1
                    validator_exchange_rate: 1,
                },
            }
        };

        // This will hold a single deterministically chosen validator definition for every identity key
        // we received validator definitions for.
        let mut resolved_validator_definitions: BTreeMap<IdentityKey, ValidatorDefinition> =
            BTreeMap::new();
        // Any conflicts in validator definitions added to the pending block need to be resolved.
        // TODO: this code should be tested to ensure changes don't break the ordering.
        for (ik, defs) in self.validator_definitions.iter_mut() {
            // Ensure the definitions are sorted by descending sequence number
            defs.sort_by(|a, b| {
                b.validator
                    .sequence_number
                    .cmp(&a.validator.sequence_number)
            });

            if defs.len() == 1 {
                // If there was only one definition for an identity key, use it.
                resolved_validator_definitions.insert(ik.clone(), defs[0].clone());
                continue;
            }

            // Sort the validator definitions into buckets by their sequence number.
            let mut new_validator_definitions_by_seq: Vec<(u32, Vec<ValidatorDefinition>)> =
                Vec::new();
            for def in defs.iter() {
                let seq = def.validator.sequence_number;
                let def = def.clone();

                // If we haven't seen this sequence number before, create a new bucket.
                if !new_validator_definitions_by_seq
                    .iter()
                    .any(|(s, _)| *s == seq)
                {
                    new_validator_definitions_by_seq.push((seq, vec![def]));
                } else {
                    // Otherwise, add the definition to the existing bucket.
                    let mut found = false;
                    for (s, defs) in new_validator_definitions_by_seq.iter_mut() {
                        if *s == seq {
                            defs.push(def);
                            found = true;
                            break;
                        }
                    }
                    assert!(found);
                }
            }

            // The highest sequence number bucket wins.
            let highest_seq_bucket = &mut new_validator_definitions_by_seq[0];

            // Sort any conflicting definitions for the highest sequence number by
            // their signatures to get a deterministic ordering.
            highest_seq_bucket.1.sort_by(|a, b| {
                let a_sig = a.auth_sig.to_bytes();
                let b_sig = b.auth_sig.to_bytes();
                a_sig.cmp(&b_sig)
            });

            // Our pick will be the first definition in the bucket after sorting by signature.
            resolved_validator_definitions.insert(ik.clone(), highest_seq_bucket.1[0].clone());
        }

        // Now that we have resolved all validator definitions, we can determine the validator
        // changes that occurred in this block.
        for (ik, def) in resolved_validator_definitions.iter() {
            if self.validators().any(|v| v.borrow().identity_key == *ik) {
                // If this is an existing validator, there will need to be a database UPDATE query.
                // The existing state will be maintained but the validator configuration will change
                // to the new definition.
                // (TODO: ensure funding stream changes are properly accounted for).
                let mut validator_info = self.validator_set.get_mut(ik).unwrap();

                // Update the internal validator configuration
                validator_info.validator = def.validator.clone();

                // Add the validator to the block's updated validators list so an UPDATE query will be generated in
                // `commit_block`.
                self.updated_validators.push(validator_info.clone());
            } else {
                // Create the new validator's ValidatorInfo struct.
                // The status will default to Inactive for new validators.
                let new_validator = make_validator(def.clone());

                // Add the validator to the internal validator set.
                self.add_validator(new_validator.clone());

                // Add the validator to the block's new validators list so an INSERT query will be generated in `commit_block`.
                self.new_validators.push(new_validator);
            }
        }

        // Set `self.tm_validator_updates` to the complete set of
        // validators and voting power.
        //
        // TODO: It could be more efficient to only return the power of
        // updated validators.
        self.tm_validator_updates = self
            .validators_info()
            .map(|v| {
                let v = v.borrow();
                let power = v.status.voting_power as u32;
                let validator = &v.validator;
                let pub_key = validator.consensus_key;
                tendermint::abci::types::ValidatorUpdate {
                    pub_key,
                    power: power.into(),
                }
            })
            .collect();

        Ok(())
    }

    pub fn update_delegations(&mut self, delegation_changes: &BTreeMap<IdentityKey, i64>) {
        // Tally the delegation changes in this transaction
        for (identity_key, delegation_change) in delegation_changes {
            *self
                .delegation_changes
                .entry(identity_key.clone())
                .or_insert(0) += delegation_change;
        }
    }

    /// Called during `end_epoch`. Will calculate validator changes that can only happen during epoch changes
    /// such as rate updates.
    pub async fn end_epoch(&mut self) -> Result<()> {
        // We've finished processing the last block of `epoch`, so we've
        // crossed the epoch boundary, and (prev | current | next) are:
        let prev_epoch = &self
            .epoch
            .clone()
            .expect("epoch must already have been set");
        let current_epoch = prev_epoch.next();
        let next_epoch = current_epoch.next();
        let current_base_rate = self.reader.base_rate_data(current_epoch.index).await?;

        // steps (foreach validator):
        // - get the total token supply for the validator's delegation tokens
        // - process the updates to the token supply:
        //   - collect all delegations occurring in previous epoch and apply them (adds to supply);
        //   - collect all undelegations started in previous epoch and apply them (reduces supply);
        // - feed the updated (current) token supply into current_rates.voting_power()
        // - persist both the current voting power and the current supply
        //

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

        let chain_params = self.reader.chain_params_rx().borrow();
        let unbonding_epochs = chain_params.unbonding_epochs;
        let validator_limit = chain_params.validator_limit;
        drop(chain_params);

        let mut next_rates = Vec::new();
        let mut next_validator_statuses = Vec::new();
        let mut reward_notes = Vec::new();
        let mut supply_updates = Vec::new();

        // this is a bit complicated: because we're in the EndBlock phase, and the
        // delegations in this block have not yet been committed, we have to combine
        // the delegations in pending_block with the ones already committed to the
        // state. otherwise the delegations committed in the epoch threshold block
        // would be lost.
        let mut delegation_changes = self.reader.delegation_changes(prev_epoch.index).await?;
        for (id_key, delta) in &self.delegation_changes {
            *delegation_changes.entry(id_key.clone()).or_insert(0) += delta;
        }

        for v in &self.validator_set {
            let validator = v.1;
            let current_rate = validator.rate_data.clone();

            let mut hold_rate_constant = |current_rate: RateData| {
                // The next epoch's rate is set to the current rate.
                let mut next_rate = current_rate;
                // Since we passed the epoch boundary, `current_epoch` will begin with
                // the next `begin_block` message.
                next_rate.epoch_index = current_epoch.index;

                next_rates.push(next_rate);
                next_validator_statuses.push(validator.status.clone());
            };
            match validator.status.state {
                // if a validator is slashed, their rates are updated to include the slashing penalty
                // and then held constant.
                //
                // if a validator is slashed during the epoch transition the current epoch's rate is set
                // to the slashed value (during end_block) and in here, the next epoch's rate is held constant.
                ValidatorState::Slashed => {
                    hold_rate_constant(current_rate);
                    continue;
                }
                // if a validator isn't part of the consensus set, we do not update their rates
                ValidatorState::Inactive => {
                    hold_rate_constant(current_rate);
                    continue;
                }
                // TODO: Are unbonding validators being handled correctly here?
                // Their rates should be held constant, but (un)delegations need to be handled as well
                _ => {}
            };

            let funding_streams = self
                .reader
                .funding_streams(validator.validator.identity_key.clone())
                .await?;

            let next_rate = current_rate.next(&next_base_rate, funding_streams.as_ref());
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
            supply_updates.push((
                identity_key.delegation_token().id(),
                identity_key.delegation_token().denom(),
                delegation_token_supply,
            ));
            let voting_power = next_rate.voting_power(delegation_token_supply, &next_base_rate);

            // Default the next state to the current state from the validator state machine.
            let next_state = self
                .get_state(&identity_key)
                .cloned()
                .expect("should be able to get next validator state from state machine");

            let next_status = ValidatorStatus {
                identity_key: identity_key.clone(),
                voting_power,
                state: next_state,
            };

            // distribute validator commission
            for stream in funding_streams {
                let commission_reward_amount = stream.reward_amount(
                    delegation_token_supply,
                    &next_base_rate,
                    &current_base_rate,
                );

                reward_notes.push((commission_reward_amount, stream.address));
            }

            // rename to curr_rate so it lines up with next_rate (same # chars)
            tracing::debug!(curr_rate = ?current_rate);
            tracing::debug!(?next_rate);
            tracing::debug!(?delegation_delta);
            tracing::debug!(?delegation_token_supply);
            tracing::debug!(?next_status);

            next_rates.push(next_rate);
            next_validator_statuses.push(next_status);
        }

        // State transitions on epoch change are handled here
        // after all rates have been calculated
        //
        // TODO: this has some overlap with the logic in ValidatorStateMachine,
        // and we never end up using some of the transition methods in ValidatorStateMachine
        // and the checks there aren't enforced as a result. Due to the code architecture
        // this was easier for the time being but should probably be addressed by ditching
        // next_validator_statuses, and making all changes directly to the validator state machine,
        // and have commit_block pull statuses from the state machine rather than next_validator_statuses

        // Sort the next validator states by voting power.
        next_validator_statuses.sort_by(|a, b| a.voting_power.cmp(&b.voting_power));
        let top_validators = next_validator_statuses
            .iter()
            .take(validator_limit as usize)
            .map(|v| v.identity_key.clone())
            .collect::<Vec<_>>();
        for validator_status in &mut next_validator_statuses {
            if validator_status.state == ValidatorState::Inactive
                || matches!(
                    validator_status.state,
                    ValidatorState::Unbonding { unbonding_epoch: _ }
                )
            {
                // If an Inactive or Unbonding validator is in the top `validator_limit` based
                // on voting power and the delegation pool has a nonzero balance,
                // then the validator should be moved to the Active state.
                if top_validators.contains(&validator_status.identity_key) {
                    // TODO: How do we check the delegation pool balance here?
                    validator_status.state = ValidatorState::Active;
                }
            } else if validator_status.state == ValidatorState::Active {
                // An Active validator could also be displaced and move to the
                // Unbonding state.
                if !top_validators.contains(&validator_status.identity_key) {
                    validator_status.state = ValidatorState::Unbonding {
                        unbonding_epoch: current_epoch.index + unbonding_epochs,
                    };
                }
            }

            // An Unbonding validator can become Inactive if the unbonding period expires
            // and the validator is still in Unbonding state
            if let ValidatorState::Unbonding { unbonding_epoch } = validator_status.state {
                if unbonding_epoch <= current_epoch.index {
                    validator_status.state = ValidatorState::Inactive;
                }
            };
        }

        for supply_update in supply_updates {
            self.add_supply_update(supply_update.0, supply_update.1, supply_update.2);
        }

        tracing::debug!(?staking_token_supply);

        self.next_rates = Some(next_rates);
        self.next_base_rate = Some(next_base_rate);
        self.next_validator_statuses = next_validator_statuses;
        self.reward_notes = reward_notes;
        self.supply_updates.insert(
            *STAKING_TOKEN_ASSET_ID,
            (STAKING_TOKEN_DENOM.clone(), staking_token_supply),
        );

        Ok(())
    }

    pub fn add_supply_update(&mut self, id: Id, denom: Denom, token_supply: u64) {
        self.supply_updates.insert(id, (denom, token_supply));
    }

    // This should *only* be called during `end_block` as validators don't
    // get exposed to consensus until then.
    pub fn add_validator(&mut self, validator: ValidatorInfo) {
        self.validator_set
            .insert(validator.validator.identity_key.clone(), validator);
    }

    pub fn update_supply_for_denom(&mut self, denom: Denom, amount: u64) {
        self.supply_updates
            .entry(denom.id())
            .or_insert((denom, 0))
            .1 += amount;
    }

    pub fn add_validator_definition(&mut self, validator_definition: ValidatorDefinition) {
        let identity_key = validator_definition.validator.identity_key.clone();
        let validator_info = ValidatorInfo {
            validator: validator_definition.validator.clone(),
            // TODO: This is definitely wrong in the case of updated validator
            // definitions and would allow resetting state/rate data!
            //
            // These should be pulled from the existing validator if it exists!
            status: ValidatorStatus {
                identity_key: validator_definition.validator.identity_key.clone(),
                // Voting power for inactive validators is 0
                voting_power: 0,
                state: ValidatorState::Inactive,
            },
            rate_data: RateData {
                identity_key: validator_definition.validator.identity_key.clone(),
                epoch_index: self
                    .epoch
                    .as_ref()
                    .expect("expect epoch to be set when validator definitions are added")
                    .index,
                // Validator reward rate is held constant for inactive validators.
                // Stake committed to inactive validators earns no rewards.
                validator_reward_rate: 0,
                // Exchange rate for inactive validators is held constant
                // and starts at 1
                validator_exchange_rate: 1,
            },
        };
        // TODO: This might not be right, since the new validator definition
        // needs to be resolved during end_block. If multiple validators
        // are defined for the same sequence ID within a transaction, it
        // is possible that state could be overriden for a validator as a
        // result.
        //
        // For example:
        // Validator A is slashed during begin_block, but then an updated
        // ValidatorDefinition is submitted for Validator A in the same block.
        self.add_validator(validator_info.clone());
        self.validator_definitions
            .entry(identity_key)
            .or_insert_with(Vec::new)
            .push(validator_definition);
    }

    pub fn get_validator_info(&self, identity_key: &IdentityKey) -> Option<&ValidatorInfo> {
        self.validator_set.get(identity_key)
    }

    pub fn get_state(&self, identity_key: &IdentityKey) -> Option<&ValidatorState> {
        self.validator_set
            .get(identity_key)
            .map(|x| &x.status.state)
    }

    pub fn validators(&self) -> impl Iterator<Item = impl Borrow<&'_ Validator>> {
        self.validator_set.iter().map(|v| &v.1.validator)
    }

    pub fn validators_info(&self) -> impl Iterator<Item = impl Borrow<&'_ ValidatorInfo>> {
        self.validator_set.iter().map(|v| v.1)
    }

    /// Returns all validators that are currently in the `Slashed` state.
    pub fn slashed_validators(&self) -> impl Iterator<Item = impl Borrow<&'_ Validator>> {
        self.validator_set
            .iter()
            .filter(|v| v.1.status.state == ValidatorState::Slashed)
            .map(|v| &v.1.validator)
    }

    pub fn unslashed_validators(&self) -> impl Iterator<Item = impl Borrow<&'_ Validator>> {
        // validators: Option<impl IntoIterator<Item = impl Borrow<&'a IdentityKey>>>,
        self.validator_set
            .iter()
            // Return all validators that are *not slashed*
            .filter(|v| v.1.status.state != ValidatorState::Slashed)
            .map(|v| &v.1.validator)
    }

    /// Mark a validator as deactivated. Only validators in the unbonding state
    /// can be deactivated.
    pub fn deactivate_validator(&mut self, ck: PublicKey) -> Result<()> {
        // Don't love this clone.
        let validator = self.get_validator_by_consensus_key(&ck)?.clone();

        let current_info = self
            .get_validator_info(&validator.identity_key)
            .ok_or(anyhow::anyhow!("Validator not found in state machine"))?;
        let current_state = current_info.status.state;

        match current_state {
            ValidatorState::Unbonding { unbonding_epoch: _ } => {
                self.validator_set
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
        // Don't love this clone.
        let validator = self.get_validator_by_consensus_key(&ck)?.clone();

        let current_info = self
            .get_validator_info(&validator.identity_key)
            .ok_or(anyhow::anyhow!("Validator not found in state machine"))?;
        let current_state = current_info.status.state;

        let mut mark_active = |validator: &Validator| -> Result<()> {
            self.validator_set
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
    pub fn slash_validator(&mut self, ck: &PublicKey, slashing_penalty: u64) -> Result<()> {
        // Don't love this clone.
        let validator = self.get_validator_by_consensus_key(ck)?.clone();

        self.slashed_validators.push(validator.identity_key.clone());

        let current_info = self
            .get_validator_info(&validator.identity_key)
            .ok_or(anyhow::anyhow!("Validator not found in state machine"))?;
        let current_state = current_info.status.state;

        let mut mark_slashed = |validator: &Validator| -> Result<()> {
            self.validator_set
                .get_mut(&validator.identity_key)
                .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                .status
                .state = ValidatorState::Slashed;
            self.validator_set
                .get_mut(&validator.identity_key)
                .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                .rate_data
                // TODO: pretty sure this calculation of slashed rate is wrong
                .validator_reward_rate -= slashing_penalty;
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
                self.validator_set
                    .get_mut(&validator.identity_key)
                    .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                    .status
                    .voting_power = 0;
                self.validator_set
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
            .validator_set
            .iter()
            .find(|v| v.1.validator.consensus_key == *ck)
            .ok_or(anyhow::anyhow!("No validator found"))?;
        Ok(&validator.1.validator)
    }
}
