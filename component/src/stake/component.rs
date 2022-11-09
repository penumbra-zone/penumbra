// Implementation of a pd component for the staking system.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::Component;
use ::metrics::{decrement_gauge, gauge, increment_gauge};
use anyhow::{anyhow, Context as _, Result};
use async_trait::async_trait;
use penumbra_chain::quarantined::Slashed;
use penumbra_chain::{genesis, Epoch, NoteSource, StateReadExt as _};
use penumbra_crypto::{DelegationToken, IdentityKey, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_proto::Protobuf;
use penumbra_storage::{State, StateRead, StateTransaction, StateWrite};
use penumbra_transaction::{
    action::{Delegate, Undelegate},
    Action, Transaction,
};
use sha2::{Digest, Sha256};
use tendermint::{
    abci::{
        self,
        types::{Evidence, LastCommitInfo, ValidatorUpdate},
    },
    block, PublicKey,
};
use tracing::{instrument, Instrument};

use crate::stake::{
    metrics,
    rate::{BaseRateData, RateData},
    state_key,
    validator::{self, Validator},
    DelegationChanges, Uptime,
};

use crate::shielded_pool::{NoteManager, SupplyRead, SupplyWrite};

use super::CurrentConsensusKeys;

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
        .unwrap();

    addr
}

// Staking component
pub struct Staking {}

pub trait ValidatorUpdates: StateRead {
    /// Returns a list of validator updates to send to Tendermint.
    ///
    /// Set during `end_block`.
    fn tendermint_validator_updates(&self) -> Option<Vec<ValidatorUpdate>> {
        self.get_ephemeral(state_key::internal::stub_tendermint_validator_updates())
            .cloned()
            .unwrap_or(None)
    }
}

impl<T: StateRead + ?Sized> ValidatorUpdates for T {}

trait PutValidatorUpdates: StateWrite {
    fn put_tendermint_validator_updates(&mut self, updates: Vec<ValidatorUpdate>) {
        tracing::info!(?updates);
        self.put_ephemeral(
            state_key::internal::stub_tendermint_validator_updates().to_owned(),
            Some(updates),
        )
    }
}

impl<T: StateWrite + ?Sized> PutValidatorUpdates for T {}

#[async_trait]
trait StakingImpl: StateWriteExt {
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
            Inactive => decrement_gauge!(metrics::INACTIVE_VALIDATORS, 1.0),
            Active => decrement_gauge!(metrics::ACTIVE_VALIDATORS, 1.0),
            Disabled => decrement_gauge!(metrics::DISABLED_VALIDATORS, 1.0),
            Jailed => decrement_gauge!(metrics::JAILED_VALIDATORS, 1.0),
            Tombstoned => decrement_gauge!(metrics::TOMBSTONED_VALIDATORS, 1.0),
        };
        match new_state {
            Inactive => increment_gauge!(metrics::INACTIVE_VALIDATORS, 1.0),
            Active => increment_gauge!(metrics::ACTIVE_VALIDATORS, 1.0),
            Disabled => increment_gauge!(metrics::DISABLED_VALIDATORS, 1.0),
            Jailed => increment_gauge!(metrics::JAILED_VALIDATORS, 1.0),
            Tombstoned => increment_gauge!(metrics::TOMBSTONED_VALIDATORS, 1.0),
        };

        // Doing a single tuple match, rather than matching on substates,
        // ensures we exhaustively cover all possible state transitions.
        use validator::BondingState::*;
        use validator::State::*;
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
                )
                .await;

                // Inform tendermint that the validator is now active.
                let power = self
                    .validator_power(identity_key)
                    .await?
                    .expect("validator that became active did not have power recorded");

                // Finally, set the validator to be active.
                self.put(state_key, Active);

                // Update metrics
                gauge!(metrics::MISSED_BLOCKS, 0.0, "identity_key" => identity_key.to_string());

                tracing::debug!(?power, "validator became active");
                Ok(())
            }
            (Active, new_state @ (Inactive | Disabled)) => {
                tracing::debug!("removing validator from active set");

                // The validator's delegation pool begins unbonding.
                self.set_validator_bonding_state(
                    identity_key,
                    Unbonding {
                        unbonding_epoch: self.current_unbonding_end_epoch().await?,
                    },
                )
                .await;

                // Finally, set the validator to be inactive or disabled.
                self.put(state_key, new_state);

                // Update metrics
                gauge!(metrics::MISSED_BLOCKS, 0.0, "identity_key" => identity_key.to_string());

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
                let penalty = self.get_chain_params().await?.slashing_penalty_downtime_bps;

                // Apply the penalty to the validator's current exchange rate.
                self.apply_slashing_penalty(identity_key, penalty).await?;

                // The validator's delegation pool begins unbonding.  Jailed
                // validators are not unbonded immediately, because they need to
                // be held accountable for byzantine behavior for the entire
                // unbonding period.
                self.set_validator_bonding_state(
                    identity_key,
                    Unbonding {
                        unbonding_epoch: self.current_unbonding_end_epoch().await?,
                    },
                )
                .await;

                // Finally, set the validator to be jailed.
                self.put(state_key, Jailed);

                Ok(())
            }
            (Active | Inactive | Disabled | Jailed, Tombstoned) => {
                let penalty = self
                    .get_chain_params()
                    .await?
                    .slashing_penalty_misbehavior_bps;

                // Apply the penalty to the validator's current exchange rate.
                self.apply_slashing_penalty(identity_key, penalty).await?;

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
        // calculate rate data for next rate, move previous next rate to cur rate,
        // and save the next rate data. ensure that non-Active validators maintain constant rates.
        let mut delegations_by_validator = BTreeMap::<IdentityKey, Vec<Delegate>>::new();
        let mut undelegations_by_validator = BTreeMap::<IdentityKey, Vec<Undelegate>>::new();
        for height in epoch_to_end.start_height().value()..=epoch_to_end.end_height().value() {
            let changes = self.delegation_changes(height.try_into().unwrap()).await?;
            for d in changes.delegations {
                delegations_by_validator
                    .entry(d.validator_identity.clone())
                    .or_insert_with(Vec::new)
                    .push(d);
            }
            for u in changes.undelegations {
                undelegations_by_validator
                    .entry(u.validator_identity.clone())
                    .or_insert_with(Vec::new)
                    .push(u);
            }
        }
        tracing::debug!(
            total_delegations = ?delegations_by_validator
                .iter()
                .map(|(_, v)| v.len())
                .sum::<usize>(),
            total_undelegations = ?undelegations_by_validator
                .iter()
                .map(|(_, v)| v.len())
                .sum::<usize>(),
        );

        let chain_params = self.get_chain_params().await?;

        tracing::debug!("processing base rate");
        // We are transitioning to the next epoch, so set "cur_base_rate" to the previous "next_base_rate", and
        // update "next_base_rate".
        let current_base_rate = self.next_base_rate().await?;

        let next_base_rate = current_base_rate.next(chain_params.base_reward_rate);

        // rename to curr_rate so it lines up with next_rate (same # chars)
        tracing::debug!(curr_base_rate = ?current_base_rate);
        tracing::debug!(?next_base_rate);

        // Update the base rates in the JMT:
        self.set_base_rates(current_base_rate.clone(), next_base_rate.clone())
            .await;

        let validator_list = self.validator_list().await?;
        for v in &validator_list {
            let validator = self.validator(v).await?.ok_or_else(|| {
                anyhow::anyhow!("validator had ID in validator_list but not found in JMT")
            })?;
            // The old epoch's "next rate" is now the "current rate".
            let current_rate = self.next_validator_rate(v).await?.ok_or_else(|| {
                anyhow::anyhow!("validator had ID in validator_list but rate not found in JMT")
            })?;

            let validator_state = self.validator_state(v).await?.ok_or_else(|| {
                anyhow::anyhow!("validator had ID in validator_list but state not found in JMT")
            })?;
            tracing::debug!(?validator, "processing validator rate updates");

            let funding_streams = validator.funding_streams;

            let next_rate =
                current_rate.next(&next_base_rate, funding_streams.as_ref(), &validator_state);
            assert!(next_rate.epoch_index == epoch_to_end.index + 2);

            let total_delegations = delegations_by_validator
                .get(&validator.identity_key)
                .into_iter()
                .flat_map(|ds| ds.iter().map(|d| u64::from(d.delegation_amount)))
                .sum::<u64>();
            let total_undelegations = undelegations_by_validator
                .get(&validator.identity_key)
                .into_iter()
                .flat_map(|us| us.iter().map(|u| u64::from(u.delegation_amount)))
                .sum::<u64>();
            let delegation_delta = (total_delegations as i64) - (total_undelegations as i64);

            tracing::debug!(
                validator = ?validator.identity_key,
                total_delegations,
                total_undelegations,
                delegation_delta
            );

            let abs_unbonded_amount =
                current_rate.unbonded_amount(delegation_delta.unsigned_abs()) as i64;
            let staking_delta = if delegation_delta >= 0 {
                // Net delegation: subtract the unbonded amount from the staking token supply
                -abs_unbonded_amount
            } else {
                // Net undelegation: add the unbonded amount to the staking token supply
                abs_unbonded_amount
            };

            // update the delegation token supply in the JMT
            self.update_token_supply(&DelegationToken::from(v).id(), delegation_delta)
                .await?;
            // update the staking token supply in the JMT
            self.update_token_supply(&STAKING_TOKEN_ASSET_ID, staking_delta)
                .await?;

            let delegation_token_supply = self
                .token_supply(&DelegationToken::from(v).id())
                .await?
                .expect("delegation token should be known");

            // Calculate the voting power in the newly beginning epoch
            let voting_power =
                current_rate.voting_power(delegation_token_supply, &current_base_rate);
            tracing::debug!(?voting_power);

            // Update the state of the validator within the validator set
            // with the newly starting epoch's calculated voting rate and power.
            self.set_validator_rates(v, current_rate.clone(), next_rate.clone());
            self.set_validator_power(v, voting_power).await?;

            // Only Active validators produce commission rewards
            // The validator *may* drop out of Active state during the next epoch,
            // but the commission rewards for the ending epoch in which it was Active
            // should still be rewarded.
            if validator_state == validator::State::Active {
                // distribute validator commission
                for stream in funding_streams {
                    let commission_reward_amount = stream.reward_amount(
                        delegation_token_supply,
                        &next_base_rate,
                        &current_base_rate,
                    );

                    self.mint_note(
                        Value {
                            amount: commission_reward_amount.into(),
                            asset_id: *STAKING_TOKEN_ASSET_ID,
                        },
                        &stream.address,
                        NoteSource::FundingStreamReward {
                            epoch_index: epoch_to_end.index,
                        },
                    )
                    .await?;
                }
            }

            // rename to curr_rate so it lines up with next_rate (same # chars)
            let delegation_denom = DelegationToken::from(v).denom();
            tracing::debug!(curr_rate = ?current_rate);
            tracing::debug!(?next_rate);
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

        for v in self.validator_list().await? {
            let state = self.validator_state(&v).await?.unwrap();
            let power = self.validator_power(&v).await?.unwrap();
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
        let limit = self.get_chain_params().await?.active_validator_limit as usize;
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

        for v in self.validator_list().await? {
            let state = self.validator_bonding_state(&v).await?.unwrap();
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
        for v in self.validator_list().await?.iter() {
            let info = self
                .validator_info(v)
                .await?
                .ok_or_else(|| anyhow::anyhow!("validator missing info"))?;

            // Compute the effective power of this validator; this is the
            // validator power, clamped to zero for all non-Active validators.
            let effective_power = if info.status.state == validator::State::Active {
                let info = self
                    .validator_info(v)
                    .await?
                    .expect("validator is in state");

                info.status.voting_power
            } else {
                0
            };

            voting_power_by_consensus_key.insert(info.validator.consensus_key, effective_power);
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
            .map(|(ck, power)| ValidatorUpdate {
                pub_key: *ck,
                power: (*power).try_into().unwrap(),
            })
            .collect();
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
    async fn track_uptime(&mut self, last_commit_info: &LastCommitInfo) -> Result<()> {
        // Note: this probably isn't the correct height for the LastCommitInfo,
        // which is about the *last* commit, but at least it'll be consistent,
        // which is all we need to count signatures.
        let height = self.get_block_height().await?;
        let params = self.get_chain_params().await?;

        // Build a mapping from addresses (20-byte truncated SHA256(pubkey)) to vote statuses.
        let did_address_vote = last_commit_info
            .votes
            .iter()
            .map(|vote| (vote.validator.address, vote.signed_last_block))
            .collect::<BTreeMap<[u8; 20], bool>>();

        // Since we don't have a lookup from "addresses" to identity keys,
        // iterate over our app's validators, and match them up with the vote data.
        for v in self.validator_list().await?.iter() {
            let info = self
                .validator_info(v)
                .await?
                .ok_or_else(|| anyhow::anyhow!("validator missing info"))?;

            if info.status.state == validator::State::Active {
                // for some reason last_commit_info has truncated sha256 hashes
                let ck_bytes = info.validator.consensus_key.to_bytes();
                let addr: [u8; 20] = Sha256::digest(&ck_bytes).as_slice()[0..20]
                    .try_into()
                    .unwrap();

                let voted = did_address_vote.get(&addr).cloned().unwrap_or(false);
                let mut uptime = self
                    .validator_uptime(v)
                    .await?
                    .ok_or_else(|| anyhow!("missing uptime for active validator {}", v))?;

                tracing::debug!(
                    ?voted,
                    num_missed_blocks = ?uptime.num_missed_blocks(),
                    identity_key = ?v,
                    ?params.missed_blocks_maximum,
                    "recorded vote info"
                );
                gauge!(metrics::MISSED_BLOCKS, uptime.num_missed_blocks() as f64, "identity_key" => v.to_string());

                uptime.mark_height_as_signed(height, voted).unwrap();
                if uptime.num_missed_blocks() as u64 >= params.missed_blocks_maximum {
                    self.set_validator_state(v, validator::State::Jailed)
                        .await?;
                } else {
                    self.set_validator_uptime(v, uptime).await;
                }
            }
        }

        Ok(())
    }

    /// Add a validator during genesis, which will start in Active
    /// state with power assigned.
    async fn add_genesis_validator(
        &mut self,
        genesis_allocations: &BTreeMap<&String, u64>,
        genesis_base_rate: &BaseRateData,
        validator: Validator,
    ) -> Result<()> {
        // Delegations require knowing the rates for the
        // next epoch, so pre-populate with 0 reward => exchange rate 1 for
        // the current and next epochs.
        let cur_rate_data = RateData {
            identity_key: validator.identity_key.clone(),
            epoch_index: genesis_base_rate.epoch_index,
            validator_reward_rate: 0,
            validator_exchange_rate: 1_0000_0000, // 1 represented as 1e8
        };
        let next_rate_data = RateData {
            identity_key: validator.identity_key.clone(),
            epoch_index: genesis_base_rate.epoch_index + 1,
            validator_reward_rate: 0,
            validator_exchange_rate: 1_0000_0000, // 1 represented as 1e8
        };

        // The initial allocations to the validator are specified in `genesis_allocations`.
        // We use these to determine the initial voting power for each validator.
        let delegation_denom = DelegationToken::from(validator.identity_key.clone())
            .denom()
            .to_string();
        let total_delegation_tokens = genesis_allocations
            .get(&delegation_denom)
            .copied()
            .unwrap_or(0);
        let power = cur_rate_data.voting_power(total_delegation_tokens, genesis_base_rate);

        self.add_validator_inner(
            validator.clone(),
            cur_rate_data,
            next_rate_data,
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
        )
        .await;

        Ok(())
    }

    /// Add a validator after genesis, which will start in Inactive
    /// state with no power assigned.
    async fn add_validator(
        &mut self,
        validator: Validator,
        cur_rate_data: RateData,
        next_rate_data: RateData,
    ) -> Result<()> {
        // We explicitly do not call `update_tm_validator_power` here,
        // as a post-genesis validator should not have power reported
        // to Tendermint until it becomes Active.
        self.add_validator_inner(
            validator.clone(),
            cur_rate_data,
            next_rate_data,
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

        self.put(state_key::validator_by_id(id), validator);

        Ok(())
    }

    async fn process_evidence(&mut self, evidence: &Evidence) -> Result<()> {
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
    #[instrument(name = "staking", skip(state, app_state))]
    async fn init_chain(state: &mut StateTransaction, app_state: &genesis::AppState) {
        let starting_height = state.get_block_height().await.unwrap();
        let starting_epoch =
            Epoch::from_height(starting_height, state.get_epoch_duration().await.unwrap());
        let epoch_index = starting_epoch.index;

        // Delegations require knowing the rates for the next epoch, so
        // pre-populate with 0 reward => exchange rate 1 for the current
        // (index 0) and next (index 1) epochs for base rate data.
        let genesis_base_rate = BaseRateData {
            epoch_index,
            base_reward_rate: 0,
            base_exchange_rate: 1_0000_0000,
        };
        let next_base_rate = BaseRateData {
            epoch_index: epoch_index + 1,
            base_reward_rate: 0,
            base_exchange_rate: 1_0000_0000,
        };
        state
            .set_base_rates(genesis_base_rate.clone(), next_base_rate)
            .await;

        // Compile totals of genesis allocations by denom, which we can use
        // to compute the delegation tokens for each validator.
        let mut genesis_allocations = BTreeMap::new();
        for allocation in &app_state.allocations {
            *genesis_allocations.entry(&allocation.denom).or_insert(0) += allocation.amount;
        }

        // Add initial validators to the JMT
        // Validators are indexed in the JMT by their public key,
        // and there is a separate key containing the list of all validator keys.
        for validator in &app_state.validators {
            // Parse the proto into a domain type.
            let validator = Validator::try_from(validator.clone()).unwrap();

            state
                .add_genesis_validator(&genesis_allocations, &genesis_base_rate, validator)
                .await
                .unwrap();
        }

        // Finally, record that there were no delegations in this block, so the data
        // isn't missing when we process the first epoch transition.
        state
            .set_delegation_changes(starting_height.try_into().unwrap(), Default::default())
            .await;

        // Build the initial validator set update.
        // First, "prime" the state with an empty set, so the build_ function can read it.
        state.put(
            state_key::current_consensus_keys().to_owned(),
            CurrentConsensusKeys::default(),
        );
        state.build_tendermint_validator_updates().await.unwrap();
    }

    #[instrument(name = "staking", skip(state, begin_block))]
    async fn begin_block(state: &mut StateTransaction, begin_block: &abci::request::BeginBlock) {
        // For each validator identified as byzantine by tendermint, update its
        // state to be slashed
        for evidence in begin_block.byzantine_validators.iter() {
            state.process_evidence(evidence).await.unwrap();
        }

        state
            .track_uptime(&begin_block.last_commit_info)
            .await
            .unwrap();
    }

    #[instrument(name = "staking", skip(tx))]
    fn check_tx_stateless(tx: Arc<Transaction>) -> Result<()> {
        // Check that the transaction undelegates from at most one validator.
        let undelegation_identities = tx
            .undelegations()
            .map(|u| u.validator_identity.clone())
            .collect::<BTreeSet<_>>();

        if undelegation_identities.len() > 1 {
            return Err(anyhow!(
                "transaction contains undelegations from multiple validators: {:?}",
                undelegation_identities
            ));
        }

        // We prohibit actions other than `Spend`, `Delegate`, `Output` and `Undelegate` in
        // transactions that contain `Undelegate`, to avoid having to quarantine them.
        if undelegation_identities.len() == 1 {
            use Action::*;
            for action in tx.transaction_body().actions {
                if !matches!(action, Undelegate(_) | Delegate(_) | Spend(_) | Output(_)) {
                    return Err(anyhow::anyhow!("transaction contains an undelegation, but also contains an action other than Spend, Delegate, Output or Undelegate"));
                }
            }
        }

        // Check that validator definitions are correctly signed and well-formed:
        for definition in tx.validator_definitions() {
            let definition = validator::Definition::try_from(definition.clone())
                .context("supplied proto is not a valid definition")?;
            // First, check the signature:
            let definition_bytes = definition.validator.encode_to_vec();
            definition
                .validator
                .identity_key
                .0
                .verify(&definition_bytes, &definition.auth_sig)
                .context("validator definition signature failed to verify")?;

            // TODO(hdevalence) -- is this duplicated by the check during parsing?
            // Check that the funding streams do not exceed 100% commission (10000bps)
            let total_funding_bps = definition
                .validator
                .funding_streams
                .iter()
                .map(|fs| fs.rate_bps as u64)
                .sum::<u64>();

            if total_funding_bps > 10000 {
                return Err(anyhow::anyhow!(
                    "validator defined {} bps of funding streams, greater than 10000bps = 100%",
                    total_funding_bps
                ));
            }
        }

        Ok(())
    }

    #[instrument(name = "staking", skip(state, tx))]
    async fn check_tx_stateful(state: Arc<State>, tx: Arc<Transaction>) -> Result<()> {
        // Tally the delegations and undelegations
        let mut delegation_changes = BTreeMap::new();
        for d in tx.delegations() {
            let next_rate_data = state
                .next_validator_rate(&d.validator_identity)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("unknown validator identity {}", d.validator_identity)
                })?
                .clone();

            // Check whether the epoch is correct first, to give a more helpful
            // error message if it's wrong.
            if d.epoch_index != next_rate_data.epoch_index {
                return Err(anyhow::anyhow!(
                    "delegation was prepared for epoch {} but the next epoch is {}",
                    d.epoch_index,
                    next_rate_data.epoch_index
                ));
            }

            // Check whether the delegation is allowed
            let validator = state
                .validator(&d.validator_identity)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing definition for validator"))?;
            let validator_state = state
                .validator_state(&d.validator_identity)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing state for validator"))?;

            use validator::State::*;
            if !validator.enabled {
                return Err(anyhow::anyhow!(
                    "delegations are only allowed to enabled validators, but {} is disabled",
                    d.validator_identity,
                ));
            }
            if !matches!(validator_state, Inactive | Active) {
                return Err(anyhow::anyhow!(
                    "delegations are only allowed to active or inactive validators, but {} is in state {:?}",
                    d.validator_identity,
                    validator_state,
                ));
            }

            // For delegations, we enforce correct computation (with rounding)
            // of the *delegation amount based on the unbonded amount*, because
            // users (should be) starting with the amount of unbonded stake they
            // wish to delegate, and computing the amount of delegation tokens
            // they receive.
            //
            // The direction of the computation matters because the computation
            // involves rounding, so while both
            //
            // (unbonded amount, rates) -> delegation amount
            // (delegation amount, rates) -> unbonded amount
            //
            // should give approximately the same results, they may not give
            // exactly the same results.
            let expected_delegation_amount =
                next_rate_data.delegation_amount(d.unbonded_amount.into());

            if expected_delegation_amount == u64::from(d.delegation_amount) {
                // The delegation amount is added to the delegation token supply.
                *delegation_changes
                    .entry(d.validator_identity.clone())
                    .or_insert(0) += i64::try_from(d.delegation_amount).unwrap();
            } else {
                return Err(anyhow::anyhow!(
                    "given {} unbonded stake, expected {} delegation tokens but description produces {}",
                    d.unbonded_amount,
                    expected_delegation_amount,
                    d.delegation_amount
                ));
            }
        }
        for u in tx.undelegations() {
            let rate_data = state
                .next_validator_rate(&u.validator_identity)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("unknown validator identity {}", u.validator_identity)
                })?;

            // Check whether the epoch is correct first, to give a more helpful
            // error message if it's wrong.
            if u.epoch_index != rate_data.epoch_index {
                return Err(anyhow::anyhow!(
                    "undelegation was prepared for next epoch {} but the next epoch is {}",
                    u.epoch_index,
                    rate_data.epoch_index
                ));
            }

            // For undelegations, we enforce correct computation (with rounding)
            // of the *unbonded amount based on the delegation amount*, because
            // users (should be) starting with the amount of delegation tokens they
            // wish to undelegate, and computing the amount of unbonded stake
            // they receive.
            //
            // The direction of the computation matters because the computation
            // involves rounding, so while both
            //
            // (unbonded amount, rates) -> delegation amount
            // (delegation amount, rates) -> unbonded amount
            //
            // should give approximately the same results, they may not give
            // exactly the same results.
            let expected_unbonded_amount = rate_data.unbonded_amount(u.delegation_amount.into());

            if expected_unbonded_amount == u64::from(u.unbonded_amount) {
                // TODO: in order to have exact tracking of the token supply, we probably
                // need to change this to record the changes to the unbonded stake and
                // the delegation token separately

                // The undelegation amount is subtracted from the delegation token supply.
                *delegation_changes
                    .entry(u.validator_identity.clone())
                    .or_insert(0) -= i64::try_from(u.delegation_amount).unwrap();
            } else {
                return Err(anyhow::anyhow!(
                    "given {} delegation tokens, expected {} unbonded stake but description produces {}",
                    u.delegation_amount,
                    expected_unbonded_amount,
                    u.unbonded_amount,
                ));
            }
        }

        // Check that the sequence numbers of updated validators are correct.
        for v in tx.validator_definitions() {
            let v = validator::Definition::try_from(v.clone())
                .context("supplied proto is not a valid definition")?;

            // Check whether we are redefining an existing validator.
            if let Some(existing_v) = state.validator(&v.validator.identity_key).await? {
                // Ensure that the highest existing sequence number is less than
                // the new sequence number.
                let current_seq = existing_v.sequence_number;
                if v.validator.sequence_number <= current_seq {
                    return Err(anyhow::anyhow!(
                        "expected sequence numbers to be increasing: current sequence number is {}",
                        current_seq
                    ));
                }
            }

            // Check whether the consensus key has already been used by another validator.
            if let Some(existing_v) = state
                .validator_by_consensus_key(&v.validator.consensus_key)
                .await?
            {
                if v.validator.identity_key != existing_v.identity_key {
                    // This is a new validator definition, but the consensus
                    // key it declares is already in use by another validator.
                    //
                    // Rejecting this is important for two reasons:
                    //
                    // 1. It prevents someone from declaring an (app-level)
                    // validator that "piggybacks" on the actual behavior of someone
                    // else's validator.
                    //
                    // 2. If we submit a validator update to Tendermint that
                    // includes duplicate consensus keys, Tendermint gets confused
                    // and hangs.
                    return Err(anyhow::anyhow!(
                        "consensus key {:?} is already in use by validator {}",
                        v.validator.consensus_key,
                        existing_v.identity_key,
                    ));
                }
            }

            // the validator definition has now passed all verification checks
        }

        Ok(())
    }

    #[instrument(name = "staking", skip(state, tx))]
    async fn execute_tx(state: &mut StateTransaction, tx: Arc<Transaction>) -> Result<()> {
        // Queue any (un)delegations for processing at the next epoch boundary.
        for action in &tx.transaction_body.actions {
            match action {
                Action::Delegate(d) => {
                    tracing::debug!(?d, "queuing delegation for next epoch");
                    state.stub_push_delegation(d.clone());
                }
                Action::Undelegate(u) => {
                    tracing::debug!(?u, "queuing undelegation for next epoch");
                    state.stub_push_undelegation(u.clone());
                }
                _ => {}
            }
        }

        // The validator definitions have been completely verified, so we can add them to the JMT
        let definitions = tx.validator_definitions().map(|v| v.to_owned());
        let cur_epoch = state.get_current_epoch().await.unwrap();

        for v in definitions {
            let v = validator::Definition::try_from(v.clone())
                .expect("we already checked that this was a valid proto");
            if state
                .validator(&v.validator.identity_key)
                .await
                .unwrap()
                .is_some()
            {
                // This is an existing validator definition.
                state.update_validator(v.validator).await.unwrap();
            } else {
                // This is a new validator definition.
                // Set the default rates and state.
                let validator_key = v.validator.identity_key.clone();

                // Delegations require knowing the rates for the
                // next epoch, so pre-populate with 0 reward => exchange rate 1 for
                // the current and next epochs.
                let cur_rate_data = RateData {
                    identity_key: validator_key.clone(),
                    epoch_index: cur_epoch.index,
                    validator_reward_rate: 0,
                    validator_exchange_rate: 1_0000_0000, // 1 represented as 1e8
                };
                let next_rate_data = RateData {
                    identity_key: validator_key.clone(),
                    epoch_index: cur_epoch.index + 1,
                    validator_reward_rate: 0,
                    validator_exchange_rate: 1_0000_0000, // 1 represented as 1e8
                };

                state
                    .add_validator(v.validator.clone(), cur_rate_data, next_rate_data)
                    .await
                    .unwrap();
            }
        }

        Ok(())
    }

    #[instrument(name = "staking", skip(state, end_block))]
    async fn end_block(state: &mut StateTransaction, end_block: &abci::request::EndBlock) {
        // Write the delegation changes for this block.
        state
            .set_delegation_changes(
                end_block.height.try_into().unwrap(),
                state.stub_delegation_changes().clone(),
            )
            .await;

        // If this is an epoch boundary, updated rates need to be calculated and set.
        let cur_epoch = state.get_current_epoch().await.unwrap();
        let cur_height = state.get_block_height().await.unwrap();

        if cur_epoch.is_epoch_end(cur_height) {
            state.end_epoch(cur_epoch).await.unwrap();
        }

        state.build_tendermint_validator_updates().await.unwrap();
    }
}

/// Extension trait providing read access to staking data.
#[async_trait]
pub trait StateReadExt: StateRead {
    /// Delegation changes accumulated over the course of this block, to be
    /// persisted at the end of the block for processing at the end of the next
    /// epoch.
    fn stub_delegation_changes(&self) -> DelegationChanges {
        self.get_ephemeral(state_key::internal::stub_delegation_changes())
            .cloned()
            .unwrap_or_default()
    }

    async fn current_base_rate(&self) -> Result<BaseRateData> {
        self.get(state_key::current_base_rate())
            .await
            .map(|rate_data| rate_data.expect("rate data must be set after init_chain"))
    }

    async fn next_base_rate(&self) -> Result<BaseRateData> {
        self.get(state_key::next_base_rate())
            .await
            .map(|rate_data| rate_data.expect("rate data must be set after init_chain"))
    }

    async fn current_validator_rate(&self, identity_key: &IdentityKey) -> Result<Option<RateData>> {
        self.get(&state_key::current_rate_by_validator(identity_key))
            .await
    }

    async fn next_validator_rate(&self, identity_key: &IdentityKey) -> Result<Option<RateData>> {
        self.get(&state_key::next_rate_by_validator(identity_key))
            .await
    }

    #[instrument(skip(self))]
    async fn validator_power(&self, identity_key: &IdentityKey) -> Result<Option<u64>> {
        self.get_proto(&state_key::power_by_validator(identity_key))
            .await
    }

    async fn validator(&self, identity_key: &IdentityKey) -> Result<Option<Validator>> {
        self.get(&state_key::validator_by_id(identity_key)).await
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
        let rate_data = self.next_validator_rate(identity_key).await?;
        match (validator, status, rate_data) {
            (Some(validator), Some(status), Some(rate_data)) => Ok(Some(validator::Info {
                validator,
                status,
                rate_data,
            })),
            _ => Ok(None),
        }
    }

    async fn validator_state(
        &self,
        identity_key: &IdentityKey,
    ) -> Result<Option<validator::State>> {
        self.get(&state_key::state_by_validator(identity_key)).await
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

    async fn validator_list(&self) -> Result<Vec<IdentityKey>> {
        Ok(self
            .get(state_key::validator_list())
            .await?
            .map(|list: validator::List| list.0)
            .unwrap_or_default())
    }

    async fn delegation_changes(&self, height: block::Height) -> Result<DelegationChanges> {
        Ok(self
            .get(&state_key::delegation_changes_by_height(height.value()))
            .await?
            .ok_or_else(|| anyhow!("missing delegation changes for block {}", height))?)
    }

    async fn validator_uptime(&self, identity_key: &IdentityKey) -> Result<Option<Uptime>> {
        self.get(&state_key::uptime_by_validator(identity_key))
            .await
    }

    async fn signed_blocks_window_len(&self) -> Result<u64> {
        Ok(self.get_chain_params().await?.signed_blocks_window_len)
    }

    async fn missed_blocks_maximum(&self) -> Result<u64> {
        Ok(self.get_chain_params().await?.missed_blocks_maximum)
    }

    async fn current_unbonding_end_epoch(&self) -> Result<u64> {
        let current_epoch = self.get_current_epoch().await?;
        let unbonding_epochs = self.get_chain_params().await?.unbonding_epochs;

        Ok(current_epoch.index + unbonding_epochs)
    }

    /// Returns the epoch and identity key for quarantining a transaction, if it should be
    /// quarantined, otherwise `None`.
    async fn should_quarantine(&self, transaction: &Transaction) -> Option<(u64, IdentityKey)> {
        let validator_identity =
            transaction
                .transaction_body
                .actions
                .iter()
                .find_map(|action| {
                    if let Action::Undelegate(Undelegate {
                        validator_identity, ..
                    }) = action
                    {
                        Some(validator_identity)
                    } else {
                        None
                    }
                })?;

        let validator_bonding_state = self
            .validator_bonding_state(validator_identity)
            .await
            .expect("validator lookup in state succeeds")
            .expect("validator is present in state");

        let should_quarantine = match validator_bonding_state {
            validator::BondingState::Unbonded => None,
            validator::BondingState::Unbonding { unbonding_epoch } => {
                Some((unbonding_epoch, *validator_identity))
            }
            validator::BondingState::Bonded => {
                let unbonding_epochs = self
                    .get_chain_params()
                    .await
                    .expect("can get chain params")
                    .unbonding_epochs;
                Some((
                    self.epoch().await.unwrap().index + unbonding_epochs,
                    *validator_identity,
                ))
            }
        };
        tracing::debug!(?should_quarantine, "should quarantine");
        should_quarantine
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

/// Extension trait providing write access to staking data.
#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Delegation changes accumulated over the course of this block, to be
    /// persisted at the end of the block for processing at the end of the next
    /// epoch.
    fn put_stub_delegation_changes(&mut self, delegation_changes: DelegationChanges) {
        self.put_ephemeral(
            state_key::internal::stub_delegation_changes().to_owned(),
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
    async fn set_base_rates(&mut self, current: BaseRateData, next: BaseRateData) {
        tracing::debug!("setting base rates");
        self.put(state_key::current_base_rate().to_owned(), current);
        self.put(state_key::next_base_rate().to_owned(), next);
    }

    #[instrument(skip(self))]
    async fn set_validator_power(
        &mut self,
        identity_key: &IdentityKey,
        voting_power: u64,
    ) -> Result<()> {
        tracing::debug!("setting validator power");
        if voting_power as i64 > MAX_VOTING_POWER || (voting_power as i64) < 0 {
            return Err(anyhow::anyhow!("invalid voting power"));
        }

        self.put_proto(state_key::power_by_validator(identity_key), voting_power);

        Ok(())
    }

    #[instrument(skip(self))]
    fn set_validator_rates(
        &mut self,
        identity_key: &IdentityKey,
        current_rates: RateData,
        next_rates: RateData,
    ) {
        tracing::debug!("setting validator rates");
        self.put(
            state_key::current_rate_by_validator(identity_key),
            current_rates,
        );
        self.put(state_key::next_rate_by_validator(identity_key), next_rates);
    }

    async fn register_consensus_key(
        &mut self,
        identity_key: &IdentityKey,
        consensus_key: &PublicKey,
    ) {
        let address = validator_address(consensus_key);
        tracing::debug!(?identity_key, ?consensus_key, hash = ?hex::encode(&address), "registering consensus key");
        self.put(
            state_key::consensus_key_by_tendermint_address(&address),
            consensus_key.clone(),
        );
        self.put(
            state_key::validator_id_by_consensus_key(consensus_key),
            identity_key.clone(),
        );
    }

    async fn apply_slashing_penalty(
        &mut self,
        identity_key: &IdentityKey,
        slashing_penalty_bps: u64,
    ) -> Result<()> {
        let mut cur_rate = self
            .current_validator_rate(identity_key)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("validator to be slashed did not have current rate in JMT")
            })?;

        // Apply the slashing penalty to the current rate...
        cur_rate = cur_rate.slash(slashing_penalty_bps);
        // ...and ensure they're held constant at the penalized rate.
        let next_rate = {
            let mut rate = cur_rate.clone();
            rate.epoch_index += 1;
            rate
        };

        self.set_validator_rates(identity_key, cur_rate, next_rate);

        // Whenever a slashing penalty is applied, we need to record that the validator was slashedt
        // in this block so that the shielded pool can unschedule unquarantines
        self.record_slashing(identity_key.clone()).await?;

        Ok(())
    }

    async fn record_slashing(&mut self, identity_key: IdentityKey) -> Result<()> {
        let height = self.get_block_height().await?;
        let key = super::state_key::slashed_validators(height).to_owned();
        let mut slashed: Slashed = self.get(&key).await?.unwrap_or_default();
        slashed.validators.push(identity_key);
        self.put(key, slashed);
        Ok(())
    }

    // Used for adding a new validator to the JMT. May be either
    // Active (a genesis validator) on Inactive (a validator added
    // post-genesis).
    async fn add_validator_inner(
        &mut self,
        validator: Validator,
        current_rates: RateData,
        next_rates: RateData,
        state: validator::State,
        bonding_state: validator::BondingState,
        power: u64,
    ) -> Result<()> {
        tracing::debug!(?validator);
        let id = validator.identity_key.clone();

        self.put(state_key::validator_by_id(&id), validator.clone());
        self.register_consensus_key(&validator.identity_key, &validator.consensus_key)
            .await;
        self.register_denom(&DelegationToken::from(&id).denom())
            .await?;

        self.set_validator_rates(&id, current_rates, next_rates);

        // We can't call `set_validator_state` here because it requires an existing validator state,
        // so we manually initialize the state for new validators.
        self.put(state_key::state_by_validator(&id), state);
        self.set_validator_power(&id, power).await?;
        self.set_validator_bonding_state(&id, bonding_state).await;

        let mut validator_list = self.validator_list().await?;
        validator_list.push(id.clone());
        tracing::debug!(?validator_list);
        self.set_validator_list(validator_list).await;

        // Lastly, update metrics for the new validator.
        match state {
            validator::State::Active => {
                increment_gauge!(metrics::ACTIVE_VALIDATORS, 1.0);
            }
            validator::State::Inactive => {
                increment_gauge!(metrics::INACTIVE_VALIDATORS, 1.0);
            }
            _ => unreachable!(),
        };
        gauge!(metrics::MISSED_BLOCKS, 0.0, "identity_key" => id.to_string());

        Ok(())
    }

    async fn set_validator_list(&mut self, validators: Vec<IdentityKey>) {
        self.put(
            state_key::validator_list().to_owned(),
            validator::List(validators),
        );
    }

    async fn set_delegation_changes(&mut self, height: block::Height, changes: DelegationChanges) {
        self.put(
            state_key::delegation_changes_by_height(height.value()),
            changes,
        );
    }

    async fn set_validator_uptime(&mut self, identity_key: &IdentityKey, uptime: Uptime) {
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
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
