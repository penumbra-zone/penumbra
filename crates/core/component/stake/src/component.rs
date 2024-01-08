// Implementation of a pd component for the staking system.
use std::{
    collections::{BTreeMap, BTreeSet},
    future::Future,
    pin::Pin,
    sync::Arc,
};

pub mod metrics;
pub mod rpc;
pub use self::metrics::register_metrics;

// TODO: move into leaf submodules under component/ and re-export

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use cnidarium_component::Component;
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use penumbra_asset::{asset, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_chain::{
    component::{StateReadExt as _, StateWriteExt as _},
    Epoch,
};
use penumbra_community_pool::component::StateWriteExt as _;

use cnidarium::{StateRead, StateWrite};
use penumbra_distributions::component::StateReadExt as _;
use penumbra_num::{fixpoint::U128x128, Amount};
use penumbra_proto::{
    state::future::{DomainFuture, ProtoFuture},
    StateReadProto, StateWriteProto,
};
use penumbra_sct::CommitmentSource;
use penumbra_shielded_pool::{
    component::{NoteManager, SupplyRead, SupplyWrite},
    genesis::Content as ShieldedPoolGenesisContent,
};
use sha2::{Digest, Sha256};
use tendermint::validator::Update;
use tendermint::{
    abci::{
        self,
        types::{CommitInfo, Misbehavior},
    },
    block, PublicKey,
};
use tokio::task::JoinSet;
use tracing::{instrument, Instrument};

use crate::{
    funding_stream::Recipient,
    genesis::Content as GenesisContent,
    params::StakeParameters,
    rate::{BaseRateData, RateData},
    state_key,
    validator::{self, Validator},
    CurrentConsensusKeys, DelegationChanges, Penalty, Uptime, {DelegationToken, IdentityKey},
};
use crate::{Delegate, Undelegate};

// Max validator power is 1152921504606846975 (i64::MAX / 8)
// https://github.com/tendermint/tendermint/blob/master/types/validator_set.go#L25
const MAX_VOTING_POWER: i64 = 1152921504606846975;

/// Translates from consensus keys to the truncated sha256 hashes in last_commit_info
/// This should really be a refined type upstream, but we can't currently upstream
/// to tendermint-rs, for process reasons, and shouldn't do our own tendermint data
/// modeling, so this is an interim hack.
fn validator_address(ck: &PublicKey) -> [u8; 20] {
    let ck_bytes = ck.to_bytes();
    let addr: [u8; 20] = Sha256::digest(&ck_bytes).as_slice()[0..20]
        .try_into()
        .expect("Sha256 digest should be 20-bytes long");

    addr
}

// Staking component
pub struct Staking {}

pub trait ValidatorUpdates: StateRead {
    /// Returns a list of validator updates to send to Tendermint.
    ///
    /// Set during `end_block`.
    fn tendermint_validator_updates(&self) -> Option<Vec<Update>> {
        self.object_get(state_key::internal::stub_tendermint_validator_updates())
            .unwrap_or(None)
    }
}

impl<T: StateRead + ?Sized> ValidatorUpdates for T {}

trait PutValidatorUpdates: StateWrite {
    fn put_tendermint_validator_updates(&mut self, updates: Vec<Update>) {
        tracing::debug!(?updates);
        self.object_put(
            state_key::internal::stub_tendermint_validator_updates(),
            Some(updates),
        )
    }
}

impl<T: StateWrite + ?Sized> PutValidatorUpdates for T {}

#[async_trait]
pub(crate) trait StakingImpl: StateWriteExt {
    /// Updates the state of the given validator, performing all necessary state transitions.
    ///
    /// This method errors on illegal state transitions; since execution must be infallible,
    /// it's the caller's responsibility to ensure that the state transitions are legal.
    async fn set_validator_state(
        &mut self,
        identity_key: &IdentityKey,
        new_state: validator::State,
    ) -> Result<()> {
        let cur_state = self.validator_state(identity_key).await?.ok_or_else(|| {
            anyhow::anyhow!("validator to have state change did not have state in JMT")
        })?;

        // Delegating to an inner method here lets us create a span that has both states,
        // without having to manage span entry/exit in async code.
        self.set_validator_state_inner(identity_key, cur_state, new_state)
            .await
    }

    // Inner function pretends to be the outer one, so we can include cur_state
    // in the tracing span.  This way, we don't need to include any information
    // in tracing events inside the function about what the state transition is,
    // because it's already attached to the span.
    #[instrument(skip(self), name = "set_validator_state")]
    async fn set_validator_state_inner(
        &mut self,
        identity_key: &IdentityKey,
        cur_state: validator::State,
        new_state: validator::State,
    ) -> Result<()> {
        let state_key = state_key::state_by_validator(identity_key).to_owned();

        // Update metrics
        match cur_state {
            Inactive => metrics::decrement_gauge!(metrics::INACTIVE_VALIDATORS, 1.0),
            Active => metrics::decrement_gauge!(metrics::ACTIVE_VALIDATORS, 1.0),
            Disabled => metrics::decrement_gauge!(metrics::DISABLED_VALIDATORS, 1.0),
            Jailed => metrics::decrement_gauge!(metrics::JAILED_VALIDATORS, 1.0),
            Tombstoned => metrics::decrement_gauge!(metrics::TOMBSTONED_VALIDATORS, 1.0),
        };
        match new_state {
            Inactive => metrics::increment_gauge!(metrics::INACTIVE_VALIDATORS, 1.0),
            Active => metrics::increment_gauge!(metrics::ACTIVE_VALIDATORS, 1.0),
            Disabled => metrics::increment_gauge!(metrics::DISABLED_VALIDATORS, 1.0),
            Jailed => metrics::increment_gauge!(metrics::JAILED_VALIDATORS, 1.0),
            Tombstoned => metrics::increment_gauge!(metrics::TOMBSTONED_VALIDATORS, 1.0),
        };

        // Doing a single tuple match, rather than matching on substates,
        // ensures we exhaustively cover all possible state transitions.
        use validator::BondingState::*;
        use validator::State::*;

        // Whenever a validator transitions out of the active state to a disabled, jailed, or
        // tombstoned state, this means that we need to explicitly signal the end of an epoch,
        // because there has been a change to the validator set outside of a normal epoch
        // transition. All other validator state transitions (including from active to inactive) are
        // triggered by epoch transitions themselves, or don't immediately affect the active
        // validator set.
        if let (Active, Disabled | Jailed | Tombstoned) = (cur_state, new_state) {
            self.signal_end_epoch();
        }

        match (cur_state, new_state) {
            (Inactive, Inactive) => Ok(()), // no-op
            (Disabled, Disabled) => Ok(()), // no-op
            (Active, Active) => Ok(()),     // no-op
            (Inactive, Active) => {
                // The validator's delegation pool becomes bonded.
                self.set_validator_bonding_state(identity_key, Bonded).await;

                // Start tracking the validator's uptime with a new uptime tracker.
                // This overwrites any existing uptime tracking, regardless of whether
                // the validator was recently in the active set.
                self.set_validator_uptime(
                    identity_key,
                    Uptime::new(
                        self.get_block_height().await?,
                        self.signed_blocks_window_len().await? as usize,
                    ),
                );

                // Inform tendermint that the validator is now active.
                let power = self
                    .validator_power(identity_key)
                    .await?
                    .expect("validator that became active did not have power recorded");

                // Finally, set the validator to be active.
                self.put(state_key, Active);

                // Update metrics
                metrics::gauge!(metrics::MISSED_BLOCKS, 0.0, "identity_key" => identity_key.to_string());

                tracing::debug!(?power, "validator became active");
                Ok(())
            }
            (Active, new_state @ (Inactive | Disabled)) => {
                tracing::debug!("removing validator from active set");

                // The validator's delegation pool begins unbonding.
                self.set_validator_bonding_state(
                    identity_key,
                    Unbonding {
                        unbonding_epoch: self.current_unbonding_end_epoch_for(identity_key).await?,
                    },
                )
                .await;

                // Finally, set the validator to be inactive or disabled.
                self.put(state_key, new_state);

                // Update metrics
                metrics::gauge!(metrics::MISSED_BLOCKS, 0.0, "identity_key" => identity_key.to_string());

                Ok(())
            }
            (Jailed, Inactive) => {
                // We don't really have to do anything here; the validator was already
                // slashed, and we're just allowing it to return to society.
                tracing::debug!("releasing validator from jail");
                self.put(state_key, Inactive);

                Ok(())
            }
            (Disabled, Inactive) => {
                // We don't really have to do anything here; we're just
                // recording that the validator was enabled.
                tracing::debug!("enabling validator");
                self.put(state_key, Inactive);

                Ok(())
            }
            (Inactive | Jailed, Disabled) => {
                // We don't really have to do anything here; we're just
                // recording that the validator was disabled, so delegations to
                // it are not allowed.
                tracing::debug!("disabling validator");
                self.put(state_key, Disabled);

                Ok(())
            }
            (Active, Jailed) => {
                let penalty = self.get_stake_params().await?.slashing_penalty_downtime;

                // Record the slashing penalty on this validator.
                self.record_slashing_penalty(identity_key, Penalty::from_bps_squared(penalty))
                    .await?;

                // The validator's delegation pool begins unbonding.  Jailed
                // validators are not unbonded immediately, because they need to
                // be held accountable for byzantine behavior for the entire
                // unbonding period.
                self.set_validator_bonding_state(
                    identity_key,
                    Unbonding {
                        unbonding_epoch: self.current_unbonding_end_epoch_for(identity_key).await?,
                    },
                )
                .await;

                // Finally, set the validator to be jailed.
                self.put(state_key, Jailed);

                Ok(())
            }
            (Active | Inactive | Disabled | Jailed, Tombstoned) => {
                let penalty = self.get_stake_params().await?.slashing_penalty_misbehavior;

                // Record the slashing penalty on this validator.
                self.record_slashing_penalty(identity_key, Penalty::from_bps_squared(penalty))
                    .await?;

                // Regardless of its current bonding state, the validator's
                // delegation pool is unbonded immediately, because the
                // validator has already had the maximum slashing penalty
                // applied.
                self.set_validator_bonding_state(identity_key, Unbonded)
                    .await;

                // Finally, set the validator to be tombstoned.
                self.put(state_key, Tombstoned);

                Ok(())
            }
            (Jailed | Disabled, Active) => {
                Err(anyhow::anyhow!("only inactive validator may become active"))
            }
            (Inactive | Jailed | Disabled, Jailed) => {
                Err(anyhow::anyhow!("only active validators may become jailed"))
            }
            (Tombstoned, Inactive | Active | Jailed | Tombstoned | Disabled) => {
                Err(anyhow::anyhow!("tombstoning is forever"))
            }
        }
    }

    #[instrument(skip(self, epoch_to_end), fields(index = epoch_to_end.index))]
    async fn end_epoch(&mut self, epoch_to_end: Epoch) -> Result<()> {
        // Collect all the delegation changes that occurred in the epoch we are ending.
        let mut delegations_by_validator = BTreeMap::<IdentityKey, Vec<Delegate>>::new();
        let mut undelegations_by_validator = BTreeMap::<IdentityKey, Vec<Undelegate>>::new();

        let end_height = self.get_block_height().await?;

        for height in epoch_to_end.start_height..=end_height {
            let changes = self
                .delegation_changes(
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
        let prev_base_rate = self.current_base_rate().await?;

        tracing::debug!(
            "fetching the issuance budget for this epoch from the distributions component"
        );

        // Fetch the issuance budget for the epoch we are ending.
        let issuance_budget_for_epoch = self
            .get_staking_token_issuance_for_epoch()
            .expect("issuance budget is always set by the distributions component");

        tracing::debug!(
            ?issuance_budget_for_epoch,
            "calculating base rate for next epoch from issuance budget"
        );

        // Compute the base reward rate for the upcoming epoch based on the total amount
        // of active stake and the issuance budget given to us by the distribution component.
        tracing::debug!("processing base rate");
        let bps_squared = Amount::from(1_0000_0000u128);
        let issuance_budget_for_epoch_bps = issuance_budget_for_epoch * bps_squared;

        let total_active_stake_previous_epoch = self.total_active_stake().await?;
        let base_reward_rate: Amount = U128x128::ratio(
            issuance_budget_for_epoch_bps,
            total_active_stake_previous_epoch,
        )
        .expect("total active stake is nonzero")
        .round_down()
        .try_into()
        .expect("rounded to an integral value");

        // TODO(erwan): use fixnum and amounts. Tracked in #3453.
        let next_base_rate = prev_base_rate.next(base_reward_rate.value() as u64);
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

        let validator_list = self.validator_list().await?;
        for validator in &validator_list {
            // Grab the current validator state.
            let validator_state = self
                .validator_state(&validator.identity_key)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("validator had ID in validator_list but state not found in JMT")
                })?;

            // We are transitioning to the next epoch, so the "current" validator
            // rate in the state is now the previous validator rate.
            let prev_validator_rate = self
                .current_validator_rate(&validator.identity_key)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("validator had ID in validator_list but current_validator_rate not found in JMT")
                })?;

            // First, apply any penalty recorded in the epoch we are ending.
            let penalty = self
                .penalty_in_epoch(&validator.identity_key, epoch_to_end.index)
                .await?
                .unwrap_or(Penalty::from_percent(0));
            let prev_validator_rate_with_penalty = prev_validator_rate.slash(penalty);

            // Then compute the next validator rate, accounting for funding streams and validator state.
            let next_validator_rate = prev_validator_rate_with_penalty.next(
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
            let delegation_delta = (total_delegations as i128) - (total_undelegations as i128);

            tracing::debug!(
                validator = ?validator.identity_key,
                total_delegations,
                total_undelegations,
                delegation_delta
            );

            // Delegations and undelegations created in the previous epoch were created
            // with the prev_validator_rate.  To compute the staking delta, we need to take
            // an absolute value and then re-apply the sign, since .unbonded_amount operates
            // on unsigned values.
            let abs_unbonded_amount =
                prev_validator_rate.unbonded_amount(delegation_delta.unsigned_abs()) as i128;
            let staking_delta = if delegation_delta >= 0 {
                // Net delegation: subtract the unbonded amount from the staking token supply
                -abs_unbonded_amount
            } else {
                // Net undelegation: add the unbonded amount to the staking token supply
                abs_unbonded_amount
            };

            // update the delegation token supply in the JMT
            self.update_token_supply(
                &DelegationToken::from(validator.identity_key).id(),
                delegation_delta,
            )
            .await?;
            // update the staking token supply in the JMT
            self.update_token_supply(&STAKING_TOKEN_ASSET_ID, staking_delta)
                .await?;

            // Get the updated delegation token supply for use calculating voting power.
            let delegation_token_supply = self
                .token_supply(&DelegationToken::from(validator.identity_key).id())
                .await?
                .expect("delegation token should be known");

            // Calculate the voting power in the newly beginning epoch
            let voting_power =
                next_validator_rate.voting_power(delegation_token_supply.into(), &next_base_rate);
            tracing::debug!(?voting_power);

            // Update the state of the validator within the validator set
            // with the newly starting epoch's calculated voting rate and power.
            self.set_validator_rates(&validator.identity_key, next_validator_rate.clone());
            self.set_validator_power(&validator.identity_key, voting_power)
                .await?;

            // If the validator is Active, distribute its funding stream rewards
            // for the preceding epoch.
            if validator_state == validator::State::Active {
                // distribute validator commission
                for stream in &validator.funding_streams {
                    let commission_reward_amount = stream.reward_amount(
                        &prev_base_rate,
                        &next_base_rate,
                        delegation_token_supply,
                    );

                    match stream.recipient() {
                        // If the recipient is an address, mint a note to that address
                        Recipient::Address(address) => {
                            self.mint_note(
                                Value {
                                    amount: commission_reward_amount.into(),
                                    asset_id: *STAKING_TOKEN_ASSET_ID,
                                },
                                &address,
                                CommitmentSource::FundingStreamReward {
                                    epoch_index: epoch_to_end.index,
                                },
                            )
                            .await?;
                        }
                        // If the recipient is the Community Pool, deposit the funds into the Community Pool
                        Recipient::CommunityPool => {
                            self.community_pool_deposit(Value {
                                amount: commission_reward_amount.into(),
                                asset_id: *STAKING_TOKEN_ASSET_ID,
                            })
                            .await?;
                        }
                    }
                }
            }

            // rename to curr_rate so it lines up with next_rate (same # chars)
            let delegation_denom = DelegationToken::from(&validator.identity_key).denom();
            tracing::debug!(?prev_validator_rate);
            tracing::debug!(?next_validator_rate);
            tracing::debug!(?delegation_delta);
            tracing::debug!(?delegation_token_supply);
            tracing::debug!(?delegation_denom);
        }

        // Now that all the voting power has been calculated for the upcoming epoch,
        // we can determine which validators are Active for the next epoch.
        self.process_validator_unbondings().await?;
        self.set_active_and_inactive_validators().await?;

        // The pending delegation changes should be empty at the beginning of the next epoch.
        // TODO: check that this was a no-op
        // self.delegation_changes = Default::default();

        Ok(())
    }

    /// Called during `end_epoch`. Will perform state transitions to validators based
    /// on changes to voting power that occurred in this epoch.
    async fn set_active_and_inactive_validators(&mut self) -> Result<()> {
        // A list of all active and inactive validators, with nonzero voting power.
        let mut validators_by_power = Vec::new();
        // A list of validators with zero power, who must be inactive.
        let mut zero_power = Vec::new();

        for v in self.validator_identity_list().await? {
            let state = self
                .validator_state(&v)
                .await?
                .context("should be able to fetch validator state")?;
            let power = self
                .validator_power(&v)
                .await?
                .context("should be able to fetch validator power")?;
            if matches!(state, validator::State::Active | validator::State::Inactive) {
                if power == 0 {
                    zero_power.push((v, power));
                } else {
                    validators_by_power.push((v, power));
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

    /// Process all validator unbondings queued for release in the current epoch.
    #[instrument(skip(self))]
    async fn process_validator_unbondings(&mut self) -> Result<()> {
        let current_epoch = self.get_current_epoch().await?;

        for v in self.validator_identity_list().await? {
            let state = self
                .validator_bonding_state(&v)
                .await?
                .context("should be able to fetch validator bonding state")?;
            if let validator::BondingState::Unbonding { unbonding_epoch } = state {
                if unbonding_epoch <= current_epoch.index {
                    self.set_validator_bonding_state(&v, validator::BondingState::Unbonded)
                        // Instrument the call with a span that includes the validator ID,
                        // since our current span doesn't have any per-validator information.
                        .instrument(tracing::debug_span!("unbonding", ?v))
                        .await;
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

        let mut voting_power_by_consensus_key = BTreeMap::<PublicKey, u64>::new();

        // First, build a mapping of consensus key to voting power for all known validators.

        // Using a JoinSet, run each validator's state queries concurrently.
        let mut js = JoinSet::new();
        for v in self.validator_identity_list().await?.iter() {
            let state = self.validator_state(v);
            let power = self.validator_power(v);
            let consensus_key = self.validator_consensus_key(v);
            js.spawn(async move {
                let state = state
                    .await?
                    .expect("every known validator must have a recorded state");
                // Compute the effective power of this validator; this is the
                // validator power, clamped to zero for all non-Active validators.
                let effective_power = if state == validator::State::Active {
                    power
                        .await?
                        .expect("every known validator must have a recorded power")
                } else {
                    0
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
            *voting_power > 0 || current_consensus_keys.contains(consensus_key)
        });

        // Finally, tell tendermint to delete any known consensus keys not otherwise updated
        for ck in current_consensus_keys.iter() {
            voting_power_by_consensus_key.entry(*ck).or_insert(0);
        }

        // Save the validator updates to send to Tendermint.
        let tendermint_validator_updates = voting_power_by_consensus_key
            .iter()
            .map(|(ck, power)| {
                Ok(Update {
                    pub_key: *ck,
                    power: (*power)
                        .try_into()
                        .context("should be able to convert u64 into validator Power")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        self.put_tendermint_validator_updates(tendermint_validator_updates);

        // Record the new consensus keys we will have told tendermint about.
        let updated_consensus_keys = CurrentConsensusKeys {
            consensus_keys: voting_power_by_consensus_key
                .iter()
                .filter_map(|(ck, power)| if *power != 0 { Some(*ck) } else { None })
                .collect(),
        };
        tracing::debug!(?updated_consensus_keys);
        self.put(
            state_key::current_consensus_keys().to_owned(),
            updated_consensus_keys,
        );

        Ok(())
    }

    #[instrument(skip(self, last_commit_info))]
    async fn track_uptime(&mut self, last_commit_info: &CommitInfo) -> Result<()> {
        // Note: this probably isn't the correct height for the LastCommitInfo,
        // which is about the *last* commit, but at least it'll be consistent,
        // which is all we need to count signatures.
        let height = self.get_block_height().await?;
        let params = self.get_stake_params().await?;

        // Build a mapping from addresses (20-byte truncated SHA256(pubkey)) to vote statuses.
        let did_address_vote = last_commit_info
            .votes
            .iter()
            .map(|vote| (vote.validator.address, vote.sig_info.is_signed()))
            .collect::<BTreeMap<[u8; 20], bool>>();

        // Since we don't have a lookup from "addresses" to identity keys,
        // iterate over our app's validators, and match them up with the vote data.
        // We can fetch all the data required for processing each validator concurrently:
        let mut js = JoinSet::new();
        for v in self.validator_identity_list().await? {
            let state = self.validator_state(&v);
            let uptime = self.validator_uptime(&v);
            let consensus_key = self.validator_consensus_key(&v);
            js.spawn(async move {
                let state = state
                    .await?
                    .expect("every known validator must have a recorded state");

                match state {
                    validator::State::Active => {
                        // If the validator is active, we need its consensus key and current uptime data:
                        Ok(Some((
                            v,
                            consensus_key
                                .await?
                                .expect("every known validator must have a recorded consensus key"),
                            uptime
                                .await?
                                .expect("every known validator must have a recorded uptime"),
                        )))
                    }
                    _ => {
                        // Otherwise, we don't need to track its uptime, and there's no data to fetch.
                        anyhow::Ok(None)
                    }
                }
            });
        }
        // Now process the data we fetched concurrently.
        // Note that this will process validator uptime changes in a random order, but because they are all
        // independent, this doesn't introduce any nondeterminism into the complete state change.
        while let Some(data) = js.join_next().await.transpose()? {
            if let Some((identity_key, consensus_key, mut uptime)) = data? {
                // for some reason last_commit_info has truncated sha256 hashes
                let addr: [u8; 20] =
                    Sha256::digest(&consensus_key.to_bytes()).as_slice()[0..20].try_into()?;

                let voted = did_address_vote
                    .get(&addr)
                    .cloned()
                    // If the height is `1`, then the `LastCommitInfo` refers to the genesis block,
                    // which has no signers -- so we'll mark all validators as having signed.
                    // https://github.com/penumbra-zone/penumbra/issues/1050
                    .unwrap_or(height == 1);

                tracing::debug!(
                    ?voted,
                    num_missed_blocks = ?uptime.num_missed_blocks(),
                    ?identity_key,
                    ?params.missed_blocks_maximum,
                    "recorded vote info"
                );
                metrics::gauge!(metrics::MISSED_BLOCKS, uptime.num_missed_blocks() as f64, "identity_key" => identity_key.to_string());

                uptime.mark_height_as_signed(height, voted)?;
                if uptime.num_missed_blocks() as u64 >= params.missed_blocks_maximum {
                    self.set_validator_state(&identity_key, validator::State::Jailed)
                        .await?;
                } else {
                    self.set_validator_uptime(&identity_key, uptime);
                }
            }
        }

        Ok(())
    }

    /// Add a validator during genesis, which will start in Active
    /// state with power assigned.
    async fn add_genesis_validator(
        &mut self,
        genesis_allocations: &BTreeMap<asset::Id, Amount>,
        genesis_base_rate: &BaseRateData,
        validator: Validator,
    ) -> Result<()> {
        let cur_rate_data = RateData {
            identity_key: validator.identity_key.clone(),
            epoch_index: genesis_base_rate.epoch_index,
            validator_reward_rate: 0,
            validator_exchange_rate: 1_0000_0000, // 1 represented as 1e8
        };

        // The initial allocations to the validator are specified in `genesis_allocations`.
        // We use these to determine the initial voting power for each validator.
        let delegation_id = DelegationToken::from(validator.identity_key.clone()).id();
        let total_delegation_tokens = genesis_allocations
            .get(&delegation_id)
            .copied()
            .unwrap_or(0u64.into());
        let power = cur_rate_data.voting_power(total_delegation_tokens.value(), genesis_base_rate);

        self.add_validator_inner(
            validator.clone(),
            cur_rate_data,
            // All genesis validators start in the "Active" state:
            validator::State::Active,
            // All genesis validators start in the "Bonded" bonding state:
            validator::BondingState::Bonded,
            power,
        )
        .await?;

        // We also need to start tracking uptime of new validators, because they
        // start in the active state, so we need to bundle in the effects of the
        // Inactive -> Active state transition.
        self.set_validator_uptime(
            &validator.identity_key,
            Uptime::new(0, self.signed_blocks_window_len().await? as usize),
        );

        Ok(())
    }

    /// Add a validator after genesis, which will start in Inactive
    /// state with no power assigned.
    async fn add_validator(&mut self, validator: Validator, cur_rate_data: RateData) -> Result<()> {
        // We explicitly do not call `update_tm_validator_power` here,
        // as a post-genesis validator should not have power reported
        // to Tendermint until it becomes Active.
        self.add_validator_inner(
            validator.clone(),
            cur_rate_data,
            // All post-genesis validators start in the "Inactive" state:
            validator::State::Inactive,
            // And post-genesis validators have "Unbonded" bonding state:
            validator::BondingState::Unbonded,
            0,
        )
        .await
    }

    // Used for updating an existing validator's definition.
    #[tracing::instrument(skip(self, validator), fields(id = ?validator.identity_key))]
    async fn update_validator(&mut self, validator: Validator) -> Result<()> {
        tracing::debug!(?validator);
        let id = &validator.identity_key;

        // Get the current state, so we can determine whether this update
        // triggers a state transition.
        let cur_state = self
            .validator_state(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("updated validator not found in JMT"))?;

        use validator::State::*;

        match (cur_state, validator.enabled) {
            (Disabled, true) => {
                // The operator has enabled their validator, so set it to Inactive.
                self.set_validator_state(id, Inactive).await?;
            }
            (Jailed, true) => {
                // Treat updates to jailed validators as unjail requests.
                self.set_validator_state(id, Inactive).await?;
            }
            (Active | Inactive | Jailed | Disabled, false) => {
                // The operator has disabled their validator.
                self.set_validator_state(id, Disabled).await?;
            }
            (Active | Inactive, true) => {
                // This validator update does not affect the validator's state.
            }
            (Tombstoned, _) => {
                // Ignore updates to tombstoned validators.
            }
        }

        // Update the consensus key lookup, in case the validator rotated their
        // consensus key.
        self.register_consensus_key(&validator.identity_key, &validator.consensus_key)
            .await;

        self.put(state_key::validators::by_id(id), validator);

        Ok(())
    }

    async fn process_evidence(&mut self, evidence: &Misbehavior) -> Result<()> {
        let validator = self
            .validator_by_tendermint_address(&evidence.validator.address)
            .await?
            .ok_or_else(|| anyhow::anyhow!("attempted to slash unknown validator"))?;

        self.set_validator_state(&validator.identity_key, validator::State::Tombstoned)
            .await
    }
}

impl<T: StateWrite + StateWriteExt + ?Sized> StakingImpl for T {}

#[async_trait]
impl Component for Staking {
    type AppState = (GenesisContent, ShieldedPoolGenesisContent);

    #[instrument(name = "staking", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(
        mut state: S,
        app_state: Option<&(GenesisContent, ShieldedPoolGenesisContent)>,
    ) {
        match app_state {
            Some((app_state, sp_app_state)) => {
                let starting_height = state
                    .get_block_height()
                    .await
                    .expect("should be able to get initial block height");
                let starting_epoch = state
                    .epoch_by_height(starting_height)
                    .await
                    .expect("should be able to get initial epoch");
                let epoch_index = starting_epoch.index;

                let genesis_base_rate = BaseRateData {
                    epoch_index,
                    base_reward_rate: 0,
                    base_exchange_rate: 1_0000_0000,
                };
                state.set_base_rate(genesis_base_rate.clone());

                // Compile totals of genesis allocations by denom, which we can use
                // to compute the delegation tokens for each validator.
                let mut genesis_allocations = BTreeMap::<_, Amount>::new();
                for allocation in &sp_app_state.allocations {
                    let value = allocation.value();
                    *genesis_allocations.entry(value.asset_id).or_default() += value.amount;
                }

                // Add initial validators to the JMT
                // Validators are indexed in the JMT by their public key,
                // and there is a separate key containing the list of all validator keys.
                for validator in &app_state.validators {
                    // Parse the proto into a domain type.
                    let validator = Validator::try_from(validator.clone())
                        .expect("should be able to parse genesis validator");

                    state
                        .add_genesis_validator(&genesis_allocations, &genesis_base_rate, validator)
                        .await
                        .expect("should be able to add genesis validator to state");
                }

                // First, "prime" the state with an empty set, so the build_ function can read it.
                state.put(
                    state_key::current_consensus_keys().to_owned(),
                    CurrentConsensusKeys::default(),
                );

                // Finally, record that there were no delegations in this block, so the data
                // isn't missing when we process the first epoch transition.
                state
                    .set_delegation_changes(
                        starting_height
                            .try_into()
                            .expect("should be able to convert u64 into block height"),
                        Default::default(),
                    )
                    .await;
            }
            None => { /* perform upgrade specific check */ }
        }
        // Build the initial validator set update.
        state
            .build_tendermint_validator_updates()
            .await
            .expect("should be able to build initial tendermint validator updates");
    }

    #[instrument(name = "staking", skip(state, begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        begin_block: &abci::request::BeginBlock,
    ) {
        let state = Arc::get_mut(state).expect("state should be unique");
        // For each validator identified as byzantine by tendermint, update its
        // state to be slashed
        for evidence in begin_block.byzantine_validators.iter() {
            state
                .process_evidence(evidence)
                .await
                .expect("should be able to process tendermint ABCI evidence");
        }

        state
            .track_uptime(&begin_block.last_commit_info)
            .await
            .expect("should be able to track uptime");
    }

    #[instrument(name = "staking", skip(state, end_block))]
    async fn end_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        end_block: &abci::request::EndBlock,
    ) {
        let state = Arc::get_mut(state).expect("state should be unique");
        // Write the delegation changes for this block.
        state
            .set_delegation_changes(
                end_block
                    .height
                    .try_into()
                    .expect("should be able to convert i64 into block height"),
                state.stub_delegation_changes().clone(),
            )
            .await;
    }

    #[instrument(name = "staking", skip(state))]
    async fn end_epoch<S: StateWrite + 'static>(state: &mut Arc<S>) -> anyhow::Result<()> {
        let state = Arc::get_mut(state).context("state should be unique")?;
        let epoch_ending = state
            .get_current_epoch()
            .await
            .context("should be able to get current epoch during end_epoch")?;
        state.end_epoch(epoch_ending).await?;
        // Since we only update the validator set at epoch boundaries,
        // we only need to build the validator set updates here in end_epoch.
        state
            .build_tendermint_validator_updates()
            .await
            .context("should be able to build tendermint validator updates")?;
        Ok(())
    }
}

/// Extension trait providing read access to staking data.
#[async_trait]
pub trait StateReadExt: StateRead {
    /// Indicates if the stake parameters have been updated in this block.
    fn stake_params_updated(&self) -> bool {
        self.object_get::<()>(state_key::stake_params_updated())
            .is_some()
    }

    /// Gets the stake parameters from the JMT.
    async fn get_stake_params(&self) -> Result<StakeParameters> {
        self.get(state_key::stake_params())
            .await?
            .ok_or_else(|| anyhow!("Missing StakeParameters"))
    }

    /// Delegation changes accumulated over the course of this block, to be
    /// persisted at the end of the block for processing at the end of the next
    /// epoch.
    fn stub_delegation_changes(&self) -> DelegationChanges {
        self.object_get(state_key::internal::stub_delegation_changes())
            .unwrap_or_default()
    }

    async fn penalty_in_epoch(
        &self,
        id: &IdentityKey,
        epoch_index: u64,
    ) -> Result<Option<Penalty>> {
        self.get(&state_key::penalty_in_epoch(id, epoch_index))
            .await
    }

    /// Returns the compounded penalty for the given validator over the half-open range of epochs [start, end).
    async fn compounded_penalty_over_range(
        &self,
        id: &IdentityKey,
        start: u64,
        end: u64,
    ) -> Result<Penalty> {
        let prefix = state_key::penalty_in_epoch_prefix(id);
        let all_penalties = self
            .prefix::<Penalty>(&prefix)
            .try_collect::<BTreeMap<String, Penalty>>()
            .await?;

        let start_key = state_key::penalty_in_epoch(id, start);
        let end_key = state_key::penalty_in_epoch(id, end);

        let mut compounded = Penalty::from_percent(0);
        for (_key, penalty) in all_penalties.range(start_key..end_key) {
            compounded = compounded.compound(*penalty);
        }

        Ok(compounded)
    }

    async fn current_base_rate(&self) -> Result<BaseRateData> {
        self.get(state_key::current_base_rate())
            .await
            .map(|rate_data| rate_data.expect("rate data must be set after init_chain"))
    }

    // This is a normal fn returning a boxed future so it can be 'static with no inferred lifetime bound
    fn current_validator_rate(
        &self,
        identity_key: &IdentityKey,
    ) -> Pin<Box<dyn Future<Output = Result<Option<RateData>>> + Send + 'static>> {
        self.get(&state_key::current_rate_by_validator(identity_key))
            .boxed()
    }

    fn validator_power(&self, identity_key: &IdentityKey) -> ProtoFuture<u64, Self::GetRawFut> {
        self.get_proto(&state_key::power_by_validator(identity_key))
    }

    async fn validator(&self, identity_key: &IdentityKey) -> Result<Option<Validator>> {
        self.get(&state_key::validators::by_id(identity_key)).await
    }

    fn validator_consensus_key(
        &self,
        identity_key: &IdentityKey,
    ) -> Pin<Box<dyn Future<Output = Result<Option<PublicKey>>> + Send + 'static>> {
        // TODO: this is pulling out the whole proto, but only parsing
        // the consensus key.  Alternatively, we could store the consensus
        // keys in a separate index.
        self.get_proto::<penumbra_proto::penumbra::core::component::stake::v1alpha1::Validator>(
            &state_key::validators::by_id(identity_key),
        )
        .map_ok(|opt| {
            opt.map(|v| {
                tendermint::PublicKey::from_raw_ed25519(v.consensus_key.as_slice())
                    .expect("validator consensus key must be valid ed25519 key")
            })
        })
        .boxed()
    }

    // Tendermint validators are referenced to us by their Tendermint consensus key,
    // but we reference them by their Penumbra identity key.
    async fn validator_by_consensus_key(&self, ck: &PublicKey) -> Result<Option<Validator>> {
        if let Some(identity_key) = self
            .get(&state_key::validator_id_by_consensus_key(ck))
            .await?
        {
            self.validator(&identity_key).await
        } else {
            return Ok(None);
        }
    }

    async fn validator_by_tendermint_address(
        &self,
        address: &[u8; 20],
    ) -> Result<Option<Validator>> {
        if let Some(consensus_key) = self
            .get(&state_key::consensus_key_by_tendermint_address(address))
            .await?
        {
            self.validator_by_consensus_key(&consensus_key).await
        } else {
            return Ok(None);
        }
    }

    async fn validator_info(&self, identity_key: &IdentityKey) -> Result<Option<validator::Info>> {
        let validator = self.validator(identity_key).await?;
        let status = self.validator_status(identity_key).await?;
        let rate_data = self.current_validator_rate(identity_key).await?;
        match (validator, status, rate_data) {
            (Some(validator), Some(status), Some(rate_data)) => Ok(Some(validator::Info {
                validator,
                status,
                rate_data,
            })),
            _ => Ok(None),
        }
    }

    fn validator_state(
        &self,
        identity_key: &IdentityKey,
    ) -> DomainFuture<validator::State, Self::GetRawFut> {
        self.get(&state_key::state_by_validator(identity_key))
    }

    async fn validator_bonding_state(
        &self,
        identity_key: &IdentityKey,
    ) -> Result<Option<validator::BondingState>> {
        self.get(&state_key::bonding_state_by_validator(identity_key))
            .await
    }

    /// Convenience method to assemble a [`ValidatorStatus`].
    async fn validator_status(
        &self,
        identity_key: &IdentityKey,
    ) -> Result<Option<validator::Status>> {
        let bonding_state = self.validator_bonding_state(identity_key).await?;
        let state = self.validator_state(identity_key).await?;
        let power = self.validator_power(identity_key).await?;
        let identity_key = identity_key.clone();
        match (state, power, bonding_state) {
            (Some(state), Some(voting_power), Some(bonding_state)) => Ok(Some(validator::Status {
                identity_key,
                state,
                voting_power,
                bonding_state,
            })),
            _ => Ok(None),
        }
    }

    /// Returns a list of all known validator identity keys.
    // TODO(erwan): #2921.
    fn validator_identity_list(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<IdentityKey>>> + Send + 'static>> {
        let mut iks = Vec::new();
        // TODO: boxing here is to avoid an Unpin problem.. should
        // we bound the StateRead stream GATs as Unpin?
        // TODO: why did the previous implementation of this method
        // fail to compile with a Self does not live longe enough error?
        let mut stream = self.prefix_keys(state_key::validators::list()).boxed();
        async move {
            while let Some(key) = stream.next().await {
                let ik = key?.as_str()[state_key::validators::list().len()..]
                    .parse::<IdentityKey>()
                    .expect("state keys should only have valid identity keys");
                iks.push(ik);
            }
            Ok(iks)
        }
        .boxed()
    }

    /// Returns a list of all known validators metadata.
    // TODO(erwan): #2921.
    async fn validator_list(&self) -> Result<Vec<Validator>> {
        self.prefix(state_key::validators::list())
            .map_ok(|(_key, validator)| validator)
            .try_collect()
            .await
    }

    async fn delegation_changes(&self, height: block::Height) -> Result<DelegationChanges> {
        Ok(self
            .get(&state_key::delegation_changes_by_height(height.value()))
            .await?
            .ok_or_else(|| anyhow!("missing delegation changes for block {}", height))?)
    }

    fn validator_uptime(
        &self,
        identity_key: &IdentityKey,
    ) -> DomainFuture<Uptime, Self::GetRawFut> {
        self.get(&state_key::uptime_by_validator(identity_key))
    }

    async fn signed_blocks_window_len(&self) -> Result<u64> {
        Ok(self.get_stake_params().await?.signed_blocks_window_len)
    }

    async fn missed_blocks_maximum(&self) -> Result<u64> {
        Ok(self.get_stake_params().await?.missed_blocks_maximum)
    }

    async fn unbonding_end_epoch_for(
        &self,
        id: &IdentityKey,
        start_epoch_index: u64,
    ) -> Result<u64> {
        let unbonding_epochs = self.get_stake_params().await?.unbonding_epochs;

        let default_unbonding = start_epoch_index + unbonding_epochs;

        let validator_unbonding =
            if let Some(validator::BondingState::Unbonding { unbonding_epoch }) =
                self.validator_bonding_state(id).await?
            {
                unbonding_epoch
            } else {
                u64::MAX
            };

        Ok(std::cmp::min(default_unbonding, validator_unbonding))
    }

    async fn current_unbonding_end_epoch_for(&self, id: &IdentityKey) -> Result<u64> {
        let current_epoch = self.get_current_epoch().await?;
        let unbonding_epochs = self.get_stake_params().await?.unbonding_epochs;

        let default_unbonding = current_epoch.index + unbonding_epochs;

        let validator_unbonding =
            if let Some(validator::BondingState::Unbonding { unbonding_epoch }) =
                self.validator_bonding_state(id).await?
            {
                unbonding_epoch
            } else {
                u64::MAX
            };

        Ok(std::cmp::min(default_unbonding, validator_unbonding))
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

/// Extension trait providing write access to staking data.
#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Writes the provided stake parameters to the JMT.
    fn put_stake_params(&mut self, params: StakeParameters) {
        // Note that the stake params have been updated:
        self.object_put(state_key::stake_params_updated(), ());

        // Change the stake parameters:
        self.put(state_key::stake_params().into(), params)
    }

    /// Delegation changes accumulated over the course of this block, to be
    /// persisted at the end of the block for processing at the end of the next
    /// epoch.
    fn put_stub_delegation_changes(&mut self, delegation_changes: DelegationChanges) {
        self.object_put(
            state_key::internal::stub_delegation_changes(),
            delegation_changes,
        )
    }

    fn stub_push_delegation(&mut self, delegation: Delegate) {
        let mut changes = self.stub_delegation_changes();
        changes.delegations.push(delegation);
        self.put_stub_delegation_changes(changes);
    }

    fn stub_push_undelegation(&mut self, undelegation: Undelegate) {
        let mut changes = self.stub_delegation_changes();
        changes.undelegations.push(undelegation);
        self.put_stub_delegation_changes(changes);
    }

    #[instrument(skip(self))]
    fn set_base_rate(&mut self, current: BaseRateData) {
        tracing::debug!("setting base rate");
        self.put(state_key::current_base_rate().to_owned(), current);
    }

    #[instrument(skip(self))]
    async fn set_validator_power(
        &mut self,
        identity_key: &IdentityKey,
        voting_power: u64,
    ) -> Result<()> {
        tracing::debug!("setting validator power");
        if voting_power as i64 > MAX_VOTING_POWER || (voting_power as i64) < 0 {
            anyhow::bail!("invalid voting power");
        }

        self.put_proto(state_key::power_by_validator(identity_key), voting_power);

        Ok(())
    }

    #[instrument(skip(self))]
    fn set_validator_rates(&mut self, identity_key: &IdentityKey, current_rates: RateData) {
        tracing::debug!("setting validator rates");
        self.put(
            state_key::current_rate_by_validator(identity_key),
            current_rates,
        );
    }

    async fn register_consensus_key(
        &mut self,
        identity_key: &IdentityKey,
        consensus_key: &PublicKey,
    ) {
        let address = validator_address(consensus_key);
        tracing::debug!(?identity_key, ?consensus_key, hash = ?hex::encode(address), "registering consensus key");
        self.put(
            state_key::consensus_key_by_tendermint_address(&address),
            consensus_key.clone(),
        );
        self.put(
            state_key::validator_id_by_consensus_key(consensus_key),
            identity_key.clone(),
        );
    }

    async fn record_slashing_penalty(
        &mut self,
        identity_key: &IdentityKey,
        slashing_penalty: Penalty,
    ) -> Result<()> {
        let current_epoch_index = self.epoch().await?.index;

        let current_penalty = self
            .penalty_in_epoch(identity_key, current_epoch_index)
            .await?
            .unwrap_or(Penalty::from_percent(0));

        let new_penalty = current_penalty.compound(slashing_penalty);

        self.put(
            state_key::penalty_in_epoch(identity_key, current_epoch_index),
            new_penalty,
        );

        Ok(())
    }

    // Used for adding a new validator to the JMT. May be either
    // Active (a genesis validator) on Inactive (a validator added
    // post-genesis).
    async fn add_validator_inner(
        &mut self,
        validator: Validator,
        current_rates: RateData,
        state: validator::State,
        bonding_state: validator::BondingState,
        power: u64,
    ) -> Result<()> {
        tracing::debug!(?validator);
        let id = validator.identity_key.clone();

        self.put(state_key::validators::by_id(&id), validator.clone());
        self.register_consensus_key(&validator.identity_key, &validator.consensus_key)
            .await;
        self.register_denom(&DelegationToken::from(&id).denom())
            .await?;

        self.set_validator_rates(&id, current_rates);

        // We can't call `set_validator_state` here because it requires an existing validator state,
        // so we manually initialize the state for new validators.
        self.put(state_key::state_by_validator(&id), state);
        self.set_validator_power(&id, power).await?;
        self.set_validator_bonding_state(&id, bonding_state).await;

        // Lastly, update metrics for the new validator.
        match state {
            validator::State::Active => {
                metrics::increment_gauge!(metrics::ACTIVE_VALIDATORS, 1.0);
            }
            validator::State::Inactive => {
                metrics::increment_gauge!(metrics::INACTIVE_VALIDATORS, 1.0);
            }
            _ => unreachable!(),
        };
        metrics::gauge!(metrics::MISSED_BLOCKS, 0.0, "identity_key" => id.to_string());

        Ok(())
    }

    async fn set_delegation_changes(&mut self, height: block::Height, changes: DelegationChanges) {
        self.put(
            state_key::delegation_changes_by_height(height.value()),
            changes,
        );
    }

    fn set_validator_uptime(&mut self, identity_key: &IdentityKey, uptime: Uptime) {
        self.put(state_key::uptime_by_validator(identity_key), uptime);
    }

    async fn set_validator_bonding_state(
        &mut self,
        identity_key: &IdentityKey,
        state: validator::BondingState,
    ) {
        tracing::debug!(?state, "set bonding state");
        self.put(state_key::bonding_state_by_validator(identity_key), state);
    }

    /// Calculate the amount of stake that is delegated to the currently active validator set,
    /// denominated in the staking token.
    #[instrument(skip(self))]
    async fn total_active_stake(&self) -> Result<Amount> {
        let validator_list = self.validator_identity_list().await?;

        let mut total_active_stake: u64 = 0;
        for validator_identity in &validator_list {
            let validator_state =
                self.validator_state(validator_identity)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "validator had ID in validator_list but state not found in JMT"
                        )
                    })?;
            if validator_state != validator::State::Active {
                continue;
            }

            let delegation_token_supply = self
                .token_supply(&DelegationToken::from(validator_identity).id())
                .await?
                .expect("delegation token should be known");

            let validator_rate = self
                .current_validator_rate(validator_identity)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("validator had ID in validator_list but rate not found in JMT")
                })?;

            // Add the validator's unbonded amount to the total active stake
            total_active_stake = total_active_stake
                .checked_add(
                    validator_rate
                        .unbonded_amount(delegation_token_supply.into())
                        .try_into()
                        .map_err(|_| {
                            anyhow::anyhow!("validator rate unbonded amount overflowed u64")
                        })?,
                )
                .ok_or_else(|| anyhow::anyhow!("total active stake overflowed u64"))?;
        }

        Ok(total_active_stake.into())
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
