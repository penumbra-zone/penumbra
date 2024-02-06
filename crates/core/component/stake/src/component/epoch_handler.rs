use penumbra_distributions::component::StateReadExt as _;
use penumbra_sct::{component::clock::EpochRead, epoch::Epoch};
use std::{
    collections::{BTreeMap, BTreeSet},
    future::Future,
    pin::Pin,
    str::FromStr,
};
use validator::BondingState::*;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use futures::{FutureExt, StreamExt, TryStreamExt};
use penumbra_asset::STAKING_TOKEN_ASSET_ID;

use cnidarium::{StateRead, StateWrite};
use penumbra_num::{fixpoint::U128x128, Amount};
use penumbra_proto::{state::future::DomainFuture, StateReadProto, StateWriteProto};
use penumbra_shielded_pool::component::{SupplyRead, SupplyWrite};
use sha2::{Digest, Sha256};
use tendermint::validator::Update;
use tendermint::{block, PublicKey};
use tokio::task::JoinSet;
use tracing::{instrument, Instrument};

use crate::{
    component::validator_manager::ValidatorManager,
    component::{
        validator_store::{ValidatorDataRead, ValidatorStore},
        PutValidatorUpdates, StakingDataRead, FP_SCALING_FACTOR,
    },
    params::StakeParameters,
    rate::{BaseRateData, RateData},
    state_key,
    validator::{self, State, Validator},
    CurrentConsensusKeys, DelegationChanges, FundingStreams, Penalty, StateReadExt, Uptime,
    {DelegationToken, IdentityKey},
};
use crate::{Delegate, Undelegate};
use once_cell::sync::Lazy;

use super::StateWriteExt;

#[async_trait]
pub trait EpochHandler: StateWriteExt {
    #[instrument(skip(self, epoch_to_end), fields(index = epoch_to_end.index))]
    /// Process the end of an epoch for the staking component.
    async fn end_epoch(&mut self, epoch_to_end: Epoch) -> Result<()> {
        let min_validator_stake = self.get_stake_params().await?.min_validator_stake;

        // Collect all the delegation changes that occurred in the epoch we are ending.
        let mut delegations_by_validator = BTreeMap::<IdentityKey, Vec<Delegate>>::new();
        let mut undelegations_by_validator = BTreeMap::<IdentityKey, Vec<Undelegate>>::new();

        let end_height = self.get_block_height().await?;

        for height in epoch_to_end.start_height..=end_height {
            let changes = self
                .get_delegation_changes(
                    height
                        .try_into()
                        .context("should be able to convert u64 into block height")?,
                )
                .await?;
            for d in changes.delegations {
                delegations_by_validator
                    .entry(d.validator_identity.clone())
                    .or_default()
                    .push(d);
            }
            for u in changes.undelegations {
                undelegations_by_validator
                    .entry(u.validator_identity.clone())
                    .or_default()
                    .push(u);
            }
        }

        tracing::debug!(
            total_delegations = ?delegations_by_validator.values().map(|v| v.len())
                .sum::<usize>(),
            total_undelegations = ?undelegations_by_validator.values().map(|v| v.len())
                .sum::<usize>(),
                epoch_start_height = epoch_to_end.start_height,
                epoch_end_height = end_height,
                "calculated delegation changes for epoch"
        );

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
        let base_reward_rate: Amount = (base_reward_rate * *FP_SCALING_FACTOR)
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
        self.set_prev_base_rate(prev_base_rate.clone());

        let mut funding_queue: Vec<(IdentityKey, FundingStreams, Amount)> = Vec::new();

        let mut validator_stream = self.consensus_set_stream()?;

        while let Some(validator_identity) = validator_stream.next().await {
            let identity_key = validator_identity?;
            let validator = self.get_validator_definition(&identity_key).await?.ok_or_else(|| {
                anyhow::anyhow!("validator (identity={}) is present in consensus set index but its definition was not found in the JMT", &identity_key)
            })?;

            // Grab the current validator state.
            let validator_state = self
                .get_validator_state(&validator.identity_key)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("validator (identity={}) is present in consensus set index but its state was not found in the JMT", &validator.identity_key)
                })?;

            // We are transitioning to the next epoch, so the "current" validator
            // rate in the state is now the previous validator rate.
            let prev_validator_rate = self
                .get_validator_rate(&validator.identity_key)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("validator (identity={}) is present in consensus set index but its rate data was not found in the JMT", &validator.identity_key)
                })?;

            // First, apply any penalty recorded in the epoch we are ending.
            let penalty = self
                .get_penalty_in_epoch(&validator.identity_key, epoch_to_end.index)
                .await?
                .unwrap_or(Penalty::from_percent(0));
            let prev_validator_rate_with_penalty = prev_validator_rate.slash(penalty);

            // Then compute the next validator rate, accounting for funding streams and validator state.
            let next_validator_rate = prev_validator_rate_with_penalty.next_epoch(
                &next_base_rate,
                validator.funding_streams.as_ref(),
                &validator_state,
            );

            let total_delegations = delegations_by_validator
                .get(&validator.identity_key)
                .into_iter()
                .flat_map(|ds| ds.iter().map(|d| d.delegation_amount.value()))
                .sum::<u128>();
            let total_undelegations = undelegations_by_validator
                .get(&validator.identity_key)
                .into_iter()
                .flat_map(|us| us.iter().map(|u| u.delegation_amount.value()))
                .sum::<u128>();

            // In theory, the maximum amount of delegation tokens is the total supply of staking tokens.
            // In practice, this is unlikely to happen, but even if it did, we anticipate that the total
            // supply of staking token is << 10^32 (2^107) tokens with a unit denomination of 10^6 (2^20),
            // so there should be ample room to cast this to an i128.
            let delegation_delta = (total_delegations as i128) - (total_undelegations as i128);

            tracing::debug!(
                validator = ?validator.identity_key,
                total_delegations,
                total_undelegations,
                delegation_delta,
                "net delegation change for validator's pool for the epoch"
            );

            // Delegations and undelegations created in the previous epoch were created
            // with the prev_validator_rate.  To compute the staking delta, we need to take
            // an absolute value and then re-apply the sign, since .unbonded_amount operates
            // on unsigned values.
            let absolute_delegation_change = Amount::from(delegation_delta.unsigned_abs());
            let absolute_unbonded_amount =
                prev_validator_rate.unbonded_amount(absolute_delegation_change);

            let delegation_token_id = DelegationToken::from(&validator.identity_key).id();

            // Staking tokens are being delegated, so the staking token supply decreases and
            // the delegation token supply increases.
            if delegation_delta > 0 {
                tracing::debug!(
                    validator = ?validator.identity_key,
                    "staking tokens are being delegated, so the staking token supply decreases and the delegation token supply increases");
                self.decrease_token_supply(&STAKING_TOKEN_ASSET_ID, absolute_unbonded_amount)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to decrease staking token supply by {}",
                            absolute_unbonded_amount
                        )
                    })?;
                self.increase_token_supply(&delegation_token_id, absolute_delegation_change)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to increase delegation token supply by {}",
                            absolute_delegation_change
                        )
                    })?;
            } else if delegation_delta < 0 {
                tracing::debug!(
                    validator = ?validator.identity_key,
                    "staking tokens are being undelegated, so the staking token supply increases and the delegation token supply decreases");
                // Vice-versa: staking tokens are being undelegated, so the staking token supply
                // increases and the delegation token supply decreases.
                self.increase_token_supply(&STAKING_TOKEN_ASSET_ID, absolute_unbonded_amount)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to increase staking token supply by {}",
                            absolute_unbonded_amount
                        )
                    })?;
                self.decrease_token_supply(&delegation_token_id, absolute_delegation_change)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to decrease delegation token supply by {}",
                            absolute_delegation_change
                        )
                    })?;
            } else {
                tracing::debug!(
                    validator = ?validator.identity_key,
                    "no change in delegation, no change in token supply")
            }

            // Get the updated delegation token supply for use calculating voting power.
            let delegation_token_supply = self
                .token_supply(&delegation_token_id)
                .await?
                .expect("delegation token should be known");

            // Calculate the voting power in the newly beginning epoch
            let voting_power = next_validator_rate.voting_power(delegation_token_supply.into());

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
            if matches!(validator_state, validator::State::Active) {
                // Here we collect funding data to create a record that the funding component
                // can "pull". We do this because by the time the funding component is executed
                // the validator set has possibly changed (e.g. a new validator enter the active
                // set).
                funding_queue.push((
                    validator.identity_key.clone(),
                    validator.funding_streams.clone(),
                    delegation_token_supply,
                ));
            }

            // We want to know if the validator has enough stake delegated to it to remain
            // in the consensus set. In order to do this, we need to know what is the "absolute"
            // (i.e. unbonded) amount corresponding to the validator's delegation pool.
            let delegation_token_denom = DelegationToken::from(&validator.identity_key).denom();
            let validator_unbonded_amount =
                next_validator_rate.unbonded_amount(delegation_token_supply);

            tracing::debug!(
                validator_identity = %validator.identity_key,
                validator_delegation_pool = ?delegation_token_supply,
                validator_unbonded_amount = ?validator_unbonded_amount,
                "calculated validator's unbonded amount for the upcoming epoch"
            );

            if validator_unbonded_amount < min_validator_stake {
                tracing::debug!(
                    validator_identity = %validator.identity_key,
                    validator_unbonded_amount = ?validator_unbonded_amount,
                    min_validator_stake = ?min_validator_stake,
                    "validator's unbonded amount is below the minimum stake threshold, transitioning to defined"
                );
                self.set_validator_state(&validator.identity_key, validator::State::Defined)
                    .await?;
            }

            tracing::debug!(validator_identity = %validator.identity_key,
                previous_epoch_validator_rate= ?prev_validator_rate,
                next_epoch_validator_rate = ?next_validator_rate,
                delegation_denom = ?delegation_token_denom,
                ?delegation_token_supply,
                "validator's end-epoch has been processed");
        }

        // We have collected the funding streams for all validators, so we can now
        // record them for the funding component to process.
        self.queue_staking_rewards(funding_queue);

        // Now that all the voting power has been calculated for the upcoming epoch,
        // we can determine which validators are Active for the next epoch.
        self.process_validator_unbondings().await?;
        self.set_active_and_inactive_validators().await?;
        Ok(())
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

    /// Process all `Unbonding` validators, transitioning them to `Unbonded` if their
    /// unbonding target has been reached.
    #[instrument(skip(self))]
    async fn process_validator_unbondings(&mut self) -> Result<()> {
        let current_epoch = self.get_current_epoch().await?;

        let mut validator_identity_stream = self.consensus_set_stream()?;
        while let Some(identity_key) = validator_identity_stream.next().await {
            let identity_key = identity_key?;
            let state = self
                .get_validator_bonding_state(&identity_key)
                .await?
                .context("should be able to fetch validator bonding state")?;

            match state {
                Bonded => continue,
                Unbonded => continue,
                Unbonding { unbonds_at_epoch } => {
                    if current_epoch.index >= unbonds_at_epoch {
                        // The validator's delegation pool has finished unbonding, so we
                        // transition it to the Unbonded state.
                        let _ = self
                            .set_validator_bonding_state(
                                &identity_key,
                                validator::BondingState::Unbonded,
                            )
                            // Instrument the call with a span that includes the validator ID,
                            // since our current span doesn't have any per-validator information.
                            .instrument(tracing::debug_span!("unbonding", ?identity_key));
                    }
                }
            }
        }

        Ok(())
    }

    /// Materializes the entire current validator set as a Tendermint update.
    ///
    /// This re-defines all validators every time, to simplify the code compared to
    /// trying to track delta updates.
    #[instrument(skip(self))]
    async fn build_tendermint_validator_updates(&mut self) -> Result<()> {
        let current_consensus_keys: CurrentConsensusKeys = self
            .get(state_key::current_consensus_keys())
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
        // were already known to Tendermint.
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
                    // Note: Since the maxim
                    power: ((*power).value() as u64).try_into()?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        self.put_tendermint_validator_updates(tendermint_validator_updates);

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
            state_key::current_consensus_keys().to_owned(),
            updated_consensus_keys,
        );

        Ok(())
    }
}

impl<T: StateWrite + StateWriteExt + ?Sized> EpochHandler for T {}
