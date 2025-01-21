use crate::{
    component::{
        stake::{
            ConsensusIndexRead, ConsensusIndexWrite, ConsensusUpdateWrite, InternalStakingData,
            RateDataWrite,
        },
        validator_handler::{
            ValidatorDataRead, ValidatorDataWrite, ValidatorManager, ValidatorPoolTracker,
        },
        SlashingData,
    },
    rate::BaseRateData,
    state_key, validator, CurrentConsensusKeys, FundingStreams, IdentityKey, Penalty, StateReadExt,
    StateWriteExt, BPS_SQUARED_SCALING_FACTOR,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use futures::{StreamExt, TryStreamExt};
use penumbra_sdk_distributions::component::StateReadExt as _;
use penumbra_sdk_num::{fixpoint::U128x128, Amount};
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};
use penumbra_sdk_sct::{component::clock::EpochRead, epoch::Epoch};
use std::collections::{BTreeMap, BTreeSet};
use tendermint::{validator::Update, PublicKey};
use tokio::task::JoinSet;
use tracing::instrument;

#[async_trait]
pub trait EpochHandler: StateWriteExt + ConsensusIndexRead {
    #[instrument(skip(self, epoch_to_end), fields(index = epoch_to_end.index))]
    /// Process the end of an epoch for the staking component.
    async fn end_epoch(&mut self, epoch_to_end: Epoch) -> Result<()> {
        // Collect all the delegation changes that occurred in the epoch we are ending.
        let mut delegations_by_validator = BTreeMap::<IdentityKey, Amount>::new();
        let mut undelegations_by_validator = BTreeMap::<IdentityKey, Amount>::new();

        let end_height = self.get_block_height().await?;
        let mut num_delegations = 0usize;
        let mut num_undelegations = 0usize;

        // Performance: see #3874.
        for height in epoch_to_end.start_height..=end_height {
            let changes = self
                .get_delegation_changes(
                    height
                        .try_into()
                        .context("should be able to convert u64 into block height")?,
                )
                .await?;

            num_delegations = num_delegations.saturating_add(changes.delegations.len());
            num_undelegations = num_undelegations.saturating_add(changes.undelegations.len());

            for d in changes.delegations {
                let validator_identity = d.validator_identity.clone();
                let delegation_tally = delegations_by_validator
                    .entry(validator_identity)
                    .or_default()
                    .saturating_add(&d.delegation_amount);
                delegations_by_validator.insert(validator_identity, delegation_tally);
            }
            for u in changes.undelegations {
                let validator_identity = u.validator_identity.clone();
                let undelegation_tally = undelegations_by_validator
                    .entry(validator_identity)
                    .or_default()
                    .saturating_add(&u.delegation_amount);

                undelegations_by_validator.insert(validator_identity, undelegation_tally);
            }
        }

        tracing::debug!(
            num_delegations,
            num_undelegations,
            epoch_start = epoch_to_end.start_height,
            epoch_end = end_height,
            epoch_index = epoch_to_end.index,
            "collected delegation changes for the epoch"
        );

        // Compute and set the chain base rate for the upcoming epoch.
        let next_base_rate = self.process_chain_base_rate().await?;

        // TODO(erwan): replace this with a tagged stream once we have tests. See #3874.
        let delegation_set = delegations_by_validator
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        let undelegation_set = undelegations_by_validator
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        let validators_with_delegation_changes = delegation_set
            .union(&undelegation_set)
            .cloned()
            .collect::<BTreeSet<_>>();

        // We're only tracking the consensus set, and each validator identity is about 64 bytes,
        // it seems reasonable to collect the entire consensus set into memory since we expect
        // less than 100k entries. We do this to keep the code simple.
        let consensus_set = self
            .consensus_set_stream()?
            .try_collect::<BTreeSet<IdentityKey>>()
            .await?;

        let validators_to_process = validators_with_delegation_changes
            .union(&consensus_set)
            .collect::<BTreeSet<_>>();

        let mut funding_queue: Vec<(IdentityKey, FundingStreams, Amount)> = Vec::new();

        for validator_identity in validators_to_process {
            let total_delegations = delegations_by_validator
                .remove(validator_identity)
                .unwrap_or_else(Amount::zero);

            let total_undelegations = undelegations_by_validator
                .remove(validator_identity)
                .unwrap_or_else(Amount::zero);

            if let Some(rewards) = self
                .process_validator(
                    validator_identity,
                    epoch_to_end,
                    next_base_rate.clone(),
                    total_delegations,
                    total_undelegations,
                )
                .await
                .map_err(|e| {
                    tracing::error!(
                        ?e,
                        ?validator_identity,
                        "failed to process validator's end-epoch"
                    );
                    e
                })?
            {
                funding_queue.push(rewards)
            }
        }

        // This is a sanity check to ensure that we have processed all the delegation changes.
        // It should be impossible for this to fail, but we check it anyway. We can remove
        // these guards when we start rolling out our testing framework and increase coverage.
        // This should coincide with a profiling/performance effort on the epoch-handler.
        assert!(delegations_by_validator.is_empty());
        assert!(undelegations_by_validator.is_empty());

        // We have collected the funding streams for all validators, so we can now
        // record them for the funding component to process.
        self.queue_staking_rewards(funding_queue);

        // Now that the consensus set voting power has been calculated, we can select the
        // top N validators to be active for the next epoch.
        self.set_active_and_inactive_validators().await?;
        Ok(())
    }

    async fn process_validator(
        &mut self,
        validator_identity: &IdentityKey,
        epoch_to_end: Epoch,
        next_base_rate: BaseRateData,
        total_delegations: Amount,
        total_undelegations: Amount,
    ) -> Result<Option<(IdentityKey, FundingStreams, Amount)>> {
        let validator = self.get_validator_definition(&validator_identity).await?.ok_or_else(|| {
            anyhow::anyhow!("validator (identity={}) is in consensus index but its definition was not found in the JMT", &validator_identity)
        })?;

        // Grab the current validator state.
        let validator_state = self
            .get_validator_state(&validator.identity_key)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("validator (identity={}) is in consensus index but its state was not found in the JMT", &validator.identity_key)
            })?;

        // We are transitioning to the next epoch, so the "current" validator
        // rate in the state is now the previous validator rate.
        let prev_validator_rate = self
            .get_validator_rate(&validator.identity_key)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("validator (identity={}) is in consensus index but its rate data was not found in the JMT", &validator.identity_key)
            })?;

        // First, apply any penalty recorded in the epoch we are ending.
        let penalty = self
            .get_penalty_in_epoch(&validator.identity_key, epoch_to_end.index)
            .await
            .unwrap_or(Penalty::from_percent(0));
        let prev_validator_rate_with_penalty = prev_validator_rate.slash(penalty);

        self.set_prev_validator_rate(
            &validator.identity_key,
            prev_validator_rate_with_penalty.clone(),
        );

        // Then compute the next validator rate, accounting for funding streams and validator state.
        let next_validator_rate = prev_validator_rate_with_penalty.next_epoch(
            &next_base_rate,
            validator.funding_streams.as_ref(),
            &validator_state,
        );

        // In theory, the maximum amount of delegation tokens is the total supply of staking tokens.
        // In practice, this is unlikely to happen, but even if it did, we anticipate that the total
        // supply of staking token is << 10^32 (2^107) tokens with a unit denomination of 10^6 (2^20),
        // so there should be ample room to cast this to an i128.
        let delegation_delta =
            (total_delegations.value() as i128) - (total_undelegations.value() as i128);

        tracing::debug!(
            validator = ?validator.identity_key,
            ?total_delegations,
            ?total_undelegations,
            delegation_delta,
            "net delegation change for validator's pool for the epoch"
        );

        let abs_delegation_change = Amount::from(delegation_delta.unsigned_abs());

        // We need to either contract or expand the validator pool size,
        // and panic if we encounter an under/overflow, because it can only
        // happen if something has gone seriously wrong with the validator rate data.
        if delegation_delta > 0 {
            self.increase_validator_pool_size(validator_identity, abs_delegation_change)
                .await
                .expect("overflow should be impossible");
        } else if delegation_delta < 0 {
            self.decrease_validator_pool_size(validator_identity, abs_delegation_change)
                .await
                .expect("underflow should be impossible");
        } else {
            tracing::debug!(
                validator = ?validator.identity_key,
                "no change in delegation, no change in token supply")
        }

        // Get the updated delegation token supply for use calculating voting power.
        let delegation_token_supply = self
            .get_validator_pool_size(validator_identity)
            .await
            .unwrap_or(Amount::zero());

        // Calculate the voting power in the newly beginning epoch
        let voting_power = next_validator_rate.voting_power(delegation_token_supply);

        tracing::debug!(
            validator = ?validator.identity_key,
            validator_delegation_pool = ?delegation_token_supply,
            validator_power = ?voting_power,
            "calculated validator's voting power for the upcoming epoch"
        );

        // Update the state of the validator within the validator set
        // with the newly starting epoch's calculated voting rate and power.
        self.set_validator_rate_data(&validator.identity_key, next_validator_rate.clone());
        self.set_validator_power(&validator.identity_key, voting_power)?;

        // The epoch is ending, so we check if this validator was active and if so
        // we queue its [`FundingStreams`] for processing by the funding component.
        let reward_queue_entry = if validator_state == validator::State::Active {
            // Here we collect funding data to create a record that the funding component
            // can "pull". We do this because by the time the funding component is executed
            // the validator set has possibly changed (e.g. a new validator enter the active
            // set).
            Some((
                validator.identity_key.clone(),
                validator.funding_streams.clone(),
                delegation_token_supply,
            ))
        } else {
            None
        };

        let final_state = self
            .try_precursor_transition(
                validator_identity,
                validator_state,
                &next_validator_rate,
                delegation_token_supply,
            )
            .await;

        tracing::debug!(validator_identity = %validator.identity_key,
            previous_epoch_validator_rate= ?prev_validator_rate,
            next_epoch_validator_rate = ?next_validator_rate,
            ?delegation_token_supply,
            voting_power = ?voting_power,
            final_state = ?final_state,
            "validator's end-epoch has been processed");

        self.process_validator_pool_state(&validator.identity_key, epoch_to_end.start_height)
            .await.map_err(|e| {
                tracing::error!(?e, validator_identity = %validator.identity_key, "failed to process validator pool state");
                e
            })?;

        // Finally, we decide whether to keep this validator in the consensus set.
        // Doing this here means that we no longer have to worry about validators
        // escaping end-epoch processing.
        //
        // Performance: NV-storage layer churn because the validator might not actually
        // be in the CS index. We should replace the union set approach with a merged
        // stream that tags items with their source. See #3874.
        if !self.belongs_in_index(&validator.identity_key).await {
            self.remove_consensus_set_index(&validator.identity_key);
        }

        Ok(reward_queue_entry)
    }

    /// Compute and return the chain base rate ("L1BOR").
    async fn process_chain_base_rate(&mut self) -> Result<BaseRateData> {
        // We are transitioning to the next epoch, so the "current" base rate in
        // the state is now the previous base rate.
        let prev_base_rate = self.get_current_base_rate().await?;

        tracing::debug!(
            "fetching the issuance budget for this epoch from the distributions component"
        );
        // Fetch the issuance budget for the epoch we are ending.
        let issuance_budget_for_epoch = self
            .get_staking_token_issuance_for_epoch()
            .expect("issuance budget is always set by the distributions component");

        // Compute the base reward rate for the upcoming epoch based on the total amount
        // of active stake and the issuance budget given to us by the distribution component.
        let total_active_stake_previous_epoch = self.total_active_stake().await?;
        tracing::debug!(
            ?total_active_stake_previous_epoch,
            ?issuance_budget_for_epoch,
            "computing base rate for the upcoming epoch"
        );

        let base_reward_rate =
            U128x128::ratio(issuance_budget_for_epoch, total_active_stake_previous_epoch)
                .expect("total active stake is nonzero");
        let base_reward_rate: Amount = (base_reward_rate * *BPS_SQUARED_SCALING_FACTOR)
            .expect("base reward rate is around one")
            .round_down()
            .try_into()
            .expect("rounded to an integral value");
        tracing::debug!(%base_reward_rate, "base reward rate for the upcoming epoch");

        let next_base_rate = prev_base_rate.next_epoch(base_reward_rate);
        tracing::debug!(
            ?prev_base_rate,
            ?next_base_rate,
            ?base_reward_rate,
            ?total_active_stake_previous_epoch,
            ?issuance_budget_for_epoch,
            "calculated base rate for the upcoming epoch"
        );

        // Set the next base rate as the new "current" base rate.
        self.set_base_rate(next_base_rate.clone());
        // We cache the previous base rate in the state, so that other components
        // can use it in their end-epoch procesisng (e.g. funding for staking rewards).
        self.set_prev_base_rate(prev_base_rate);
        Ok(next_base_rate)
    }

    /// Called during `end_epoch`. Will perform state transitions to validators based
    /// on changes to voting power that occurred in this epoch.
    async fn set_active_and_inactive_validators(&mut self) -> Result<()> {
        // A list of all active and inactive validators, with nonzero voting power.
        let mut validators_by_power = Vec::new();
        // A list of validators with zero power, who must be inactive.
        let mut zero_power = Vec::new();

        let mut validator_identity_stream = self.consensus_set_stream()?;
        while let Some(identity_key) = validator_identity_stream.next().await {
            let identity_key = identity_key?;
            let state = self
                .get_validator_state(&identity_key)
                .await?
                .context("should be able to fetch validator state")?;
            let power = self
                .get_validator_power(&identity_key)
                .await?
                .unwrap_or_default();
            if matches!(state, validator::State::Active | validator::State::Inactive) {
                if power == Amount::zero() {
                    zero_power.push((identity_key, power));
                } else {
                    validators_by_power.push((identity_key, power));
                }
            }
        }

        // Sort by voting power descending.
        validators_by_power.sort_by(|a, b| b.1.cmp(&a.1));

        // The top `limit` validators with nonzero power become active.
        // All other validators become inactive.
        let limit = self.get_stake_params().await?.active_validator_limit as usize;
        let active = validators_by_power.iter().take(limit);
        let inactive = validators_by_power
            .iter()
            .skip(limit)
            .chain(zero_power.iter());

        for (v, _) in active {
            self.set_validator_state(v, validator::State::Active)
                .await?;
        }
        for (v, _) in inactive {
            self.set_validator_state(v, validator::State::Inactive)
                .await?;
        }

        Ok(())
    }

    /// Materializes the entire current validator set as a CometBFT update.
    ///
    /// This re-defines all validators every time, to simplify the code compared to
    /// trying to track delta updates.
    #[instrument(skip(self))]
    async fn build_cometbft_validator_updates(&mut self) -> Result<()> {
        let current_consensus_keys: CurrentConsensusKeys = self
            .get(state_key::consensus_update::consensus_keys())
            .await?
            .expect("current consensus keys must be present");
        let current_consensus_keys = current_consensus_keys
            .consensus_keys
            .into_iter()
            .collect::<BTreeSet<_>>();

        let mut voting_power_by_consensus_key = BTreeMap::<PublicKey, Amount>::new();

        // First, build a mapping of consensus key to voting power for all known validators.

        // Using a JoinSet, run each validator's state queries concurrently.
        let mut js: JoinSet<std::prelude::v1::Result<(PublicKey, Amount), anyhow::Error>> =
            JoinSet::new();
        let mut validator_identity_stream = self.consensus_set_stream()?;
        while let Some(identity_key) = validator_identity_stream.next().await {
            let identity_key = identity_key?;
            let state = self.get_validator_state(&identity_key);
            let power = self.get_validator_power(&identity_key);
            let consensus_key = self.fetch_validator_consensus_key(&identity_key);
            js.spawn(async move {
                let state = state
                    .await?
                    .expect("every known validator must have a recorded state");
                // Compute the effective power of this validator; this is the
                // validator power, clamped to zero for all non-Active validators.
                let effective_power = if matches!(state, validator::State::Active) {
                    power
                        .await?
                        .expect("every active validator must have a recorded power")
                } else {
                    Amount::zero()
                };

                let consensus_key = consensus_key
                    .await?
                    .expect("every known validator must have a recorded consensus key");

                anyhow::Ok((consensus_key, effective_power))
            });
        }
        // Now collect the computed results into the lookup table.
        while let Some(pair) = js.join_next().await.transpose()? {
            let (consensus_key, effective_power) = pair?;
            voting_power_by_consensus_key.insert(consensus_key, effective_power);
        }

        // Next, filter that mapping to exclude any zero-power validators, UNLESS they
        // were already known to CometBFT.
        voting_power_by_consensus_key.retain(|consensus_key, voting_power| {
            *voting_power > Amount::zero() || current_consensus_keys.contains(consensus_key)
        });

        // Finally, tell tendermint to delete any known consensus keys not otherwise updated
        for ck in current_consensus_keys.iter() {
            voting_power_by_consensus_key
                .entry(*ck)
                .or_insert(Amount::zero());
        }

        // Save the validator updates to send to Tendermint.
        let tendermint_validator_updates = voting_power_by_consensus_key
            .iter()
            .map(|(consensus_key, power)| {
                Ok(Update {
                    pub_key: *consensus_key,
                    // Validator voting power is measured in units of staking tokens,
                    // at time, CometBFT has an upper limit of around 2^60 - 1.
                    // This means that there is an upper bound on the maximum possible issuance
                    // at around 10^12 units of staking tokens.
                    power: ((*power).value() as u64).try_into()?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        self.put_cometbft_validator_updates(tendermint_validator_updates);

        // Record the new consensus keys we will have told tendermint about.
        let updated_consensus_keys = CurrentConsensusKeys {
            consensus_keys: voting_power_by_consensus_key
                .iter()
                .filter_map(|(consensus_key, power)| {
                    if *power != Amount::zero() {
                        Some(*consensus_key)
                    } else {
                        None
                    }
                })
                .collect(),
        };
        tracing::debug!(?updated_consensus_keys);
        self.put(
            state_key::consensus_update::consensus_keys().to_owned(),
            updated_consensus_keys,
        );

        Ok(())
    }
}

impl<T: StateWrite + ConsensusIndexRead + ?Sized> EpochHandler for T {}
