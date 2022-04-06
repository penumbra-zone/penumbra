use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use itertools::Itertools;
use penumbra_proto::Protobuf;
use penumbra_stake::{
    BaseRateData, Delegate, DelegationChanges, Epoch, IdentityKey, RateData, Undelegate, Validator,
    ValidatorList, ValidatorState, STAKING_TOKEN_ASSET_ID,
};
use penumbra_transaction::{Action, Transaction};

use tendermint::{
    abci::{
        self,
        types::{Evidence, ValidatorUpdate},
    },
    block, PublicKey,
};
use tracing::instrument;

use super::{app::View as _, shielded_pool::View as _, Component, Overlay};
use crate::{genesis, WriteOverlayExt};

// Staking component
pub struct Staking {
    overlay: Overlay,
    /// Delegation changes accumulated over the course of this block, to be
    /// persisted at the end of the block for processing at the end of the next
    /// epoch.
    delegation_changes: DelegationChanges,
}

impl Staking {
    #[instrument(skip(self))]
    async fn end_epoch(&mut self, epoch_to_end: Epoch) -> Result<()> {
        // calculate rate data for next rate, move previous next rate to cur rate,
        // and save the next rate data. ensure that non-Active validators maintain constant rates.
        let mut total_changes = DelegationChanges::default();
        for height in epoch_to_end.start_height().value()..=epoch_to_end.end_height().value() {
            let changes = self
                .overlay
                .delegation_changes(height.try_into().unwrap())
                .await?;
            total_changes.delegations.extend(changes.delegations);
            total_changes.undelegations.extend(changes.undelegations);
        }
        tracing::debug!(
            total_delegations = total_changes.delegations.len(),
            total_undelegations = total_changes.undelegations.len(),
        );

        // now the delegations and undelegations need to be grouped by validator
        let delegations_by_validator: BTreeMap<IdentityKey, Vec<Delegate>> = total_changes
            .delegations
            .into_iter()
            .group_by(|d| d.validator_identity.clone())
            .into_iter()
            .map(|(k, v)| (k, v.collect()))
            .collect();
        let undelegations_by_validator: BTreeMap<IdentityKey, Vec<Undelegate>> = total_changes
            .undelegations
            .into_iter()
            .group_by(|u| u.validator_identity.clone())
            .into_iter()
            .map(|(k, v)| (k, v.collect()))
            .collect();

        let chain_params = self.overlay.get_chain_params().await?;
        let unbonding_epochs = chain_params.unbonding_epochs;
        let active_validator_limit = chain_params.active_validator_limit;

        tracing::debug!("processing base rate");
        // We are transitioning to the next epoch, so set "cur_base_rate" to the previous "next_base_rate", and
        // update "next_base_rate".
        let current_base_rate = self.overlay.next_base_rate().await?;
        /// FIXME: set this less arbitrarily, and allow this to be set per-epoch
        /// 3bps -> 11% return over 365 epochs, why not
        const BASE_REWARD_RATE: u64 = 3_0000;

        let next_base_rate = current_base_rate.next(BASE_REWARD_RATE);

        // rename to curr_rate so it lines up with next_rate (same # chars)
        tracing::debug!(curr_base_rate = ?current_base_rate);
        tracing::debug!(?next_base_rate);

        // Update the base rates in the JMT:
        self.overlay
            .set_base_rates(current_base_rate.clone(), next_base_rate.clone())
            .await;

        let validator_list = self.overlay.validator_list().await?;
        for v in &validator_list {
            let validator = self.overlay.validator(v).await?.ok_or_else(|| {
                anyhow::anyhow!("validator had ID in validator_list but not found in JMT")
            })?;
            // The old epoch's "current rate" is going to become the "previous rate".
            let prev_rate = self
                .overlay
                .current_validator_rate(v)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("validator had ID in validator_list but rate not found in JMT")
                })?;
            // And the old epoch's "next rate" is going to become the "current rate".
            let current_rate = self.overlay.next_validator_rate(v).await?.ok_or_else(|| {
                anyhow::anyhow!("validator had ID in validator_list but rate not found in JMT")
            })?;

            let validator_state = self.overlay.validator_state(v).await?.ok_or_else(|| {
                anyhow::anyhow!("validator had ID in validator_list but state not found in JMT")
            })?;
            tracing::debug!(?validator, "processing validator rate updates");

            // The "prev rate" for the validator should be for the ending epoch
            assert!(prev_rate.epoch_index == epoch_to_end.index);

            let funding_streams = validator.funding_streams;

            let next_rate =
                current_rate.next(&next_base_rate, funding_streams.as_ref(), &validator_state);
            assert!(next_rate.epoch_index == epoch_to_end.index + 1);

            let validator_delegations = delegations_by_validator
                .get(&validator.identity_key)
                .cloned()
                .unwrap_or_default();
            let validator_undelegations = undelegations_by_validator
                .get(&validator.identity_key)
                .cloned()
                .unwrap_or_default();
            let delegation_delta: i64 = validator_delegations
                .iter()
                .map(|d| d.delegation_amount)
                .sum::<u64>() as i64
                - validator_undelegations
                    .iter()
                    .map(|u| u.delegation_amount)
                    .sum::<u64>() as i64;

            let delegation_amount = delegation_delta.abs() as u64;
            let mut unbonded_amount: i64 =
                i64::try_from(current_rate.unbonded_amount(delegation_amount))?;

            // TODO: not sure if this is implemented correctly, it's quite a bit different
            // from the old implementation
            if delegation_delta > 0 {
                // net delegation: subtract the unbonded amount from the staking token supply
                unbonded_amount *= -1;
            } else {
                // net undelegation: add the unbonded amount to the staking token supply
            }

            // update the delegation token supply in the JMT
            self.overlay
                .update_token_supply(&v.delegation_token().id(), delegation_delta)
                .await?;
            // update the staking token supply in the JMT
            self.overlay
                .update_token_supply(&STAKING_TOKEN_ASSET_ID, unbonded_amount)
                .await?;

            let delegation_token_supply = self
                .overlay
                .token_supply(&v.delegation_token().id())
                .await?
                .expect("delegation token should be known");

            // Calculate the voting power in the newly beginning epoch
            let voting_power =
                current_rate.voting_power(delegation_token_supply, &current_base_rate);
            tracing::debug!(?voting_power);

            // Update the status of the validator within the validator set
            // with the newly starting epoch's calculated voting rate and power.
            self.overlay
                .set_validator_rates(v, current_rate.clone(), next_rate.clone())
                .await;
            self.overlay.set_validator_power(v, voting_power).await;

            // Only Active validators produce commission rewards
            // The validator *may* drop out of Active state during the next epoch,
            // but the commission rewards for the ending epoch in which it was Active
            // should still be rewarded.
            if validator_state == ValidatorState::Active {
                // distribute validator commission
                for stream in funding_streams {
                    let commission_reward_amount = stream.reward_amount(
                        delegation_token_supply,
                        &next_base_rate,
                        &current_base_rate,
                    );

                    // TODO: Unclear how to tell the shielded pool we need to mint
                    // a note here. Maybe set it on the JMT and deal with it over there?
                    // reward_notes.push((commission_reward_amount, stream.address));
                }
            }

            // rename to curr_rate so it lines up with next_rate (same # chars)
            let delegation_denom = v.delegation_token().denom();
            tracing::debug!(curr_rate = ?current_rate);
            tracing::debug!(?next_rate);
            tracing::debug!(?delegation_delta);
            tracing::debug!(?delegation_token_supply);
            tracing::debug!(?delegation_denom);
        }

        // Now that all the voting power has been calculated for the upcoming epoch,
        // we can determine which validators are Active for the next epoch.
        self.process_epoch_transitions(epoch_to_end, active_validator_limit, unbonding_epochs)
            .await?;

        Ok(())
    }

    /// Called during `end_epoch`. Will perform state transitions to validators based
    /// on changes to voting power that occurred in this epoch.
    pub async fn process_epoch_transitions(
        &mut self,
        epoch_to_end: Epoch,
        active_validator_limit: u64,
        unbonding_epochs: u64,
    ) -> Result<()> {
        // Sort the next validator states by voting power.
        struct VPower {
            identity_key: IdentityKey,
            power: u64,
            state: ValidatorState,
        };

        let mut validator_power_list = Vec::new();
        for v in self.overlay.validator_list().await?.iter() {
            let power = self
                .overlay
                .validator_power(v)
                .await?
                .ok_or_else(|| anyhow::anyhow!("validator missing power"))?;
            let state = self
                .overlay
                .validator_state(v)
                .await?
                .ok_or_else(|| anyhow::anyhow!("validator missing state"))?;
            validator_power_list.push(VPower {
                identity_key: v.clone(),
                power,
                state,
            });
        }

        // Sort by voting power
        validator_power_list.sort_by(|a, b| a.power.cmp(&b.power));

        // Grab the top `active_validator_limit` validators
        let top_validators = validator_power_list
            .iter()
            .take(active_validator_limit as usize)
            .map(|v| v.identity_key.clone())
            .collect::<Vec<_>>();

        // Iterate every validator and update according to their state and voting power.
        for vp in &validator_power_list {
            if vp.state == ValidatorState::Inactive
                || matches!(vp.state, ValidatorState::Unbonding { unbonding_epoch: _ })
            {
                // If an Inactive or Unbonding validator is in the top `active_validator_limit` based
                // on voting power and the delegation pool has a nonzero balance (meaning non-zero voting power),
                // then the validator should be moved to the Active state.
                if top_validators.contains(&vp.identity_key) && vp.power > 0 {
                    self.overlay
                        .set_validator_state(&vp.identity_key, ValidatorState::Active)
                        .await;
                }
            } else if vp.state == ValidatorState::Active {
                // An Active validator could also be displaced and move to the
                // Unbonding state.
                if !top_validators.contains(&vp.identity_key) {
                    // Unbonding the validator means that it can no longer participate
                    // in consensus, so its voting power is set to 0.
                    self.overlay.set_validator_power(&vp.identity_key, 0).await;
                    self.overlay
                        .set_validator_state(
                            &vp.identity_key,
                            ValidatorState::Unbonding {
                                unbonding_epoch: unbonding_epochs,
                            },
                        )
                        .await;
                }
            }

            // An Unbonding validator can become Inactive if the unbonding period expires
            // and the validator is still in Unbonding state
            if let ValidatorState::Unbonding { unbonding_epoch } = vp.state {
                if unbonding_epoch <= epoch_to_end.index {
                    self.overlay
                        .set_validator_state(&vp.identity_key, ValidatorState::Inactive)
                        .await;
                }
            };
        }

        Ok(())
    }

    // Returns the list of validator updates formatted for inclusion in the Tendermint `EndBlockResponse`
    pub async fn tm_validator_updates(&self) -> Result<Vec<ValidatorUpdate>> {
        // Return the voting power for all known validators.
        // This isn't strictly necessary because tendermint technically expects
        // an update, however it is useful for debugging.
        let mut updates = Vec::new();
        for v in self.overlay.validator_list().await?.iter() {
            let power = self
                .overlay
                .validator_power(v)
                .await?
                .ok_or_else(|| anyhow::anyhow!("validator missing power"))?;
            let validator = self
                .overlay
                .validator(&v)
                .await?
                .ok_or_else(|| anyhow::anyhow!("validator missing"))?;
            updates.push(ValidatorUpdate {
                pub_key: validator.consensus_key.clone(),
                power: power.try_into()?,
            });
        }

        Ok(updates)
    }
}

#[async_trait]
impl Component for Staking {
    async fn new(overlay: Overlay) -> Result<Self> {
        Ok(Self {
            overlay,
            delegation_changes: Default::default(),
        })
    }

    async fn init_chain(&mut self, app_state: &genesis::AppState) -> Result<()> {
        let starting_height = self.overlay.get_block_height().await?;
        let starting_epoch =
            Epoch::from_height(starting_height, self.overlay.get_epoch_duration().await?);
        let epoch_index = starting_epoch.index;

        // Delegations require knowing the rates for the next epoch, so
        // pre-populate with 0 reward => exchange rate 1 for the current
        // (index 0) and next (index 1) epochs for base rate data.
        let cur_base_rate = BaseRateData {
            epoch_index,
            base_reward_rate: 0,
            base_exchange_rate: 1_0000_0000,
        };
        let next_base_rate = BaseRateData {
            epoch_index: epoch_index + 1,
            base_reward_rate: 0,
            base_exchange_rate: 1_0000_0000,
        };
        self.overlay
            .set_base_rates(cur_base_rate, next_base_rate)
            .await;

        // Add initial validators to the JMT
        // Validators are indexed in the JMT by their public key,
        // and there is a separate key containing the list of all validator keys.
        let mut validator_list = Vec::new();
        for validator in &app_state.validators {
            let validator_key = validator.validator.identity_key.clone();

            // Delegations require knowing the rates for the
            // next epoch, so pre-populate with 0 reward => exchange rate 1 for
            // the current and next epochs.
            let cur_rate_data = RateData {
                identity_key: validator_key.clone(),
                epoch_index,
                validator_reward_rate: 0,
                validator_exchange_rate: 1_0000_0000, // 1 represented as 1e8
            };
            let next_rate_data = RateData {
                identity_key: validator_key.clone(),
                epoch_index: epoch_index + 1,
                validator_reward_rate: 0,
                validator_exchange_rate: 1_0000_0000, // 1 represented as 1e8
            };

            self.overlay
                .save_validator(
                    validator.validator.clone(),
                    cur_rate_data,
                    next_rate_data,
                    // All genesis validators start in the "Active" state:
                    ValidatorState::Active,
                )
                .await;
            validator_list.push(validator_key);
        }

        self.overlay.set_validator_list(validator_list).await;

        // Finally, record that there were no delegations in this block, so the data
        // isn't missing when we process the first epoch transition.
        self.overlay
            .set_delegation_changes(0u32.into(), Default::default())
            .await;

        Ok(())
    }

    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock) -> Result<()> {
        tracing::debug!("Staking: begin_block");

        // For each validator identified as byzantine by tendermint, update its
        // state to be slashed.
        for evidence in begin_block.byzantine_validators.iter() {
            self.overlay.slash_validator(evidence).await?;
        }

        Ok(())
    }

    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        // Check that the transaction undelegates from at most one validator.
        let undelegation_identities = tx
            .undelegations()
            .map(|u| u.validator_identity.clone())
            .collect::<BTreeSet<_>>();

        if undelegation_identities.len() > 1 {
            return Err(anyhow!(
                "Transaction contains undelegations from multiple validators: {:?}",
                undelegation_identities
            ));
        }

        // Check that validator definitions are correctly signed and well-formed:
        for definition in tx.validator_definitions() {
            // First, check the signature:
            let definition_bytes = definition.validator.encode_to_vec();
            definition
                .validator
                .identity_key
                .0
                .verify(&definition_bytes, &definition.auth_sig)
                .context("Validator definition signature failed to verify")?;

            // Check that the funding streams do not exceed 100% commission (10000bps)
            let total_funding_bps = definition
                .validator
                .funding_streams
                .iter()
                .map(|fs| fs.rate_bps as u64)
                .sum::<u64>();

            if total_funding_bps > 10000 {
                return Err(anyhow::anyhow!(
                    "Validator defined {} bps of funding streams, greater than 10000bps = 100%",
                    total_funding_bps
                ));
            }
        }

        Ok(())
    }

    async fn check_tx_stateful(&self, _tx: &Transaction) -> Result<()> {
        Ok(())
    }

    async fn execute_tx(&mut self, tx: &Transaction) -> Result<()> {
        // Queue any (un)delegations for processing at the next epoch boundary.
        for action in &tx.transaction_body.actions {
            match action {
                Action::Delegate(d) => {
                    tracing::debug!(?d, "queuing delegation for next epoch");
                    self.delegation_changes.delegations.push(d.clone());
                }
                Action::Undelegate(u) => {
                    tracing::debug!(?u, "queuing undelegation for next epoch");
                    self.delegation_changes.undelegations.push(u.clone());
                }
                _ => {}
            }
        }

        // TODO: process validator definitions

        Ok(())
    }

    async fn end_block(&mut self, end_block: &abci::request::EndBlock) -> Result<()> {
        // Write the delegation changes for this block.
        self.overlay
            .set_delegation_changes(
                end_block.height.try_into().unwrap(),
                std::mem::take(&mut self.delegation_changes),
            )
            .await;

        // If this is an epoch boundary, updated rates need to be calculated and set.
        let cur_epoch = self.overlay.get_current_epoch().await?;
        let cur_height = self.overlay.get_block_height().await?;

        if cur_epoch.is_epoch_end(cur_height) {
            self.end_epoch(cur_epoch).await?;
        }

        Ok(())
    }
}

/// Extension trait providing read/write access to staking data.
///
/// TODO: should this be split into Read and Write traits?
#[async_trait]
pub trait View: WriteOverlayExt {
    async fn current_base_rate(&self) -> Result<BaseRateData> {
        self.get_domain("staking/base_rate/current".into())
            .await
            .map(|rate_data| rate_data.expect("rate data must be set after init_chain"))
    }

    async fn next_base_rate(&self) -> Result<BaseRateData> {
        self.get_domain("staking/base_rate/next".into())
            .await
            .map(|rate_data| rate_data.expect("rate data must be set after init_chain"))
    }

    #[instrument(skip(self))]
    async fn set_base_rates(&self, current: BaseRateData, next: BaseRateData) {
        tracing::debug!("setting base rates");
        self.put_domain("staking/base_rate/current".into(), current)
            .await;
        self.put_domain("staking/base_rate/next".into(), next).await;
    }

    async fn current_validator_rate(&self, identity_key: &IdentityKey) -> Result<Option<RateData>> {
        self.get_domain(format!("staking/validators/{}/rate/current", identity_key).into())
            .await
    }

    async fn next_validator_rate(&self, identity_key: &IdentityKey) -> Result<Option<RateData>> {
        self.get_domain(format!("staking/validators/{}/rate/next", identity_key).into())
            .await
    }

    #[instrument(skip(self))]
    async fn set_validator_power(&self, identity_key: &IdentityKey, voting_power: u64) {
        tracing::debug!("setting validator power");
        self.put_proto(
            format!("staking/validators/{}/power", identity_key).into(),
            voting_power,
        )
        .await;
    }

    #[instrument(skip(self))]
    async fn validator_power(&self, identity_key: &IdentityKey) -> Result<Option<u64>> {
        tracing::debug!("getting validator power");
        self.get_proto(format!("staking/validators/{}/power", identity_key).into())
            .await
    }

    #[instrument(skip(self))]
    async fn set_validator_rates(
        &self,
        identity_key: &IdentityKey,
        current_rates: RateData,
        next_rates: RateData,
    ) {
        tracing::debug!("setting validator rates");
        self.put_domain(
            format!("staking/validators/{}/rate/current", identity_key).into(),
            current_rates,
        )
        .await;
        self.put_domain(
            format!("staking/validators/{}/rate/next", identity_key).into(),
            next_rates,
        )
        .await;
    }

    #[instrument(skip(self))]
    async fn set_validator_state(&self, identity_key: &IdentityKey, state: ValidatorState) {
        tracing::debug!("setting validator state");
        self.put_domain(
            format!("staking/validators/{}/state", identity_key).into(),
            state,
        )
        .await;
    }

    async fn validator(&self, identity_key: &IdentityKey) -> Result<Option<Validator>> {
        self.get_domain(format!("staking/validators/{}", identity_key).into())
            .await
    }

    // Tendermint validators are referenced to us by their Tendermint consensus key,
    // but we reference them by their Penumbra identity key.
    async fn validator_by_consensus_key(&self, ck: &PublicKey) -> Result<Option<Validator>> {
        // We maintain an internal mapping of consensus keys to identity keys to make this
        // lookup more efficient.
        let identity_key: Option<IdentityKey> = self
            .get_domain(format!("staking/consensus_key/{}", ck.to_hex()).into())
            .await?;

        if identity_key.is_none() {
            return Ok(None);
        }

        let identity_key = identity_key.unwrap();

        self.validator(&identity_key).await
    }

    async fn slash_validator(&mut self, evidence: &Evidence) -> Result<()> {
        let ck = tendermint::PublicKey::from_raw_ed25519(&evidence.validator.address)
            .ok_or_else(|| anyhow::anyhow!("invalid ed25519 consensus pubkey from tendermint"))
            .unwrap();

        let validator = self
            .validator_by_consensus_key(&ck)
            .await?
            .ok_or_else(|| anyhow::anyhow!("attempted to slash validator not found in JMT"))?;

        let slashing_penalty = self.get_chain_params().await?.slashing_penalty;

        tracing::info!(?validator, ?slashing_penalty, "slashing validator");

        let cur_state = self
            .validator_state(&validator.identity_key)
            .await?
            .ok_or_else(|| anyhow::anyhow!("validator to be slashed did not have state in JMT"))?;

        // Ensure that the state transitions are valid.
        match cur_state {
            ValidatorState::Active => {}
            ValidatorState::Unbonding { unbonding_epoch: _ } => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "only validators in the active or unbonding state may be slashed"
                ))
            }
        };

        // Mark the state as "slashed" in the JMT, and apply the slashing penalty.
        self.set_validator_state(&validator.identity_key, ValidatorState::Slashed);

        let mut cur_rate = self
            .current_validator_rate(&validator.identity_key)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("validator to be slashed did not have current rate in JMT")
            })?;

        cur_rate = cur_rate.slash(slashing_penalty);

        // TODO: would it be better to call `current_base_rate.next`? the same logic exists
        // within there, but it requires passing in the current base rates & funding streams,
        // which aren't actually used because the rate is held constant. So, doing it this way
        // avoids a couple unnecessary JMT reads that the `current_base_rate.next` API would require.
        //
        // At any rate, the next rate is held constant for slashed validators.
        let mut next_rate = cur_rate.clone();
        next_rate.epoch_index += 1;

        self.set_validator_rates(&validator.identity_key, cur_rate, next_rate)
            .await;

        Ok(())
    }

    // TODO(zbuc): does this make sense? will we always be saving cur&next rate data when we save a validator definition?
    async fn save_validator(
        &self,
        validator: Validator,
        current_rates: RateData,
        next_rates: RateData,
        state: ValidatorState,
    ) {
        tracing::debug!(?validator);
        let id = validator.identity_key.clone();
        self.put_domain(format!("staking/validators/{}", id).into(), validator)
            .await;
        self.set_validator_rates(&id, current_rates, next_rates)
            .await;
        self.set_validator_state(&id, state).await;
    }

    async fn validator_state(&self, identity_key: &IdentityKey) -> Result<Option<ValidatorState>> {
        self.get_domain(format!("staking/validators/{}/state", identity_key).into())
            .await
    }

    async fn validator_list(&self) -> Result<Vec<IdentityKey>> {
        Ok(self
            .get_domain("staking/validators/list".into())
            .await?
            .map(|list: ValidatorList| list.0)
            .unwrap_or_default())
    }

    async fn set_validator_list(&self, validators: Vec<IdentityKey>) {
        self.put_domain("staking/validators/list".into(), ValidatorList(validators))
            .await;
    }

    async fn delegation_changes(&self, height: block::Height) -> Result<DelegationChanges> {
        Ok(self
            .get_domain(format!("staking/delegation_changes/{}", height.value()).into())
            .await?
            .ok_or_else(|| anyhow!("missing delegation changes for block {}", height))?)
    }

    async fn set_delegation_changes(&self, height: block::Height, changes: DelegationChanges) {
        self.put_domain(
            format!("staking/delegation_changes/{}", height.value()).into(),
            changes,
        )
        .await
    }
}

impl<T: WriteOverlayExt + Send + Sync> View for T {}
