use std::collections::BTreeMap;

use crate::{
    component::{
        metrics,
        stake::{ConsensusIndexRead, ConsensusIndexWrite, RateDataWrite},
        validator_handler::ValidatorDataWrite,
    },
    event,
    rate::{BaseRateData, RateData},
    validator::{State, Validator},
    DelegationToken,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt as _;
use penumbra_num::Amount;
use penumbra_sct::component::clock::{EpochManager, EpochRead};
use penumbra_shielded_pool::component::{SupplyRead as _, SupplyWrite};
use sha2::{Digest as _, Sha256};
use tendermint::abci::types::{CommitInfo, Misbehavior};
use tokio::task::JoinSet;
use validator::BondingState::*;

use cnidarium::StateWrite;
use penumbra_proto::StateWriteProto;
use tracing::instrument;

use crate::{
    component::validator_handler::ValidatorDataRead,
    component::StateReadExt as _,
    component::StateWriteExt as _,
    state_key,
    validator::{self},
    IdentityKey, Penalty, Uptime,
};
use penumbra_asset::asset;

#[async_trait]
pub trait ValidatorManager: StateWrite {
    /// Perform a state transition for the specified validator and new state.
    /// Initial validator state is defined using [`add_validator`]
    ///                                                                      
    ///    ┌───────────────────────────────────────────────────────────┐     
    ///    │      ┌─────────────────────────────────┐                  │     
    ///    │      │             ┌───────────────────┼──────────────┐   │     
    ///    │      │             ▼                   │              │   │     
    ///    ▼      ▼        ┌────────┐          ┌────────┐          │   │     
    /// ┌────────────┐     │        │◀────────▶│        │        ┌────────┐  
    /// │  Defined   │◀───▶│Inactive│          │ Active │───────▶│ Jailed │─┐
    /// └────────────┘     │        │  ┌───────│        │        └────────┘ │
    ///        ▲           └────────┘  │       └────────┘             │     │
    ///        │                ▲      │            │                 │     │
    ///        │                │      │            │                 │     │
    ///        │                │      │            ▼                 │     │
    ///        │                │      │      ┌──────────┐            │     │
    ///        │                │      │      │Tombstoned│◀───────────┘     │
    ///        │                │      │      └──────────┘                  │
    ///        │                ▼      ▼                                    │
    ///        │            ┌──────────────┐                                │
    ///        └───────────▶│   Disabled   │◀───────────────────────────────┘
    ///                     └──────────────┘                                                                                
    /// # Errors
    /// This method errors on illegal state transitions; since execution must be infallible,
    /// it's the caller's responsibility to ensure that the state transitions are legal.
    ///
    /// It can also error if the validator is not found in the state, though this should
    /// never happen.
    async fn set_validator_state(
        &mut self,
        identity_key: &IdentityKey,
        new_state: validator::State,
    ) -> Result<()> {
        let old_state = self
            .get_validator_state(identity_key)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("validator state not found for validator {}", identity_key)
            })?;

        // Delegating to an inner method here lets us create a span that has both states,
        // without having to manage span entry/exit in async code.
        self.set_validator_state_inner(identity_key, old_state, new_state)
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
        old_state: validator::State,
        new_state: validator::State,
    ) -> Result<()> {
        use validator::State::*;
        let validator_state_path = state_key::validators::state::by_id(identity_key);

        // Validator state transitions are usually triggered by an epoch transition. The exception
        // to this rule is when a validator exits the active set. In this case, we want to end the
        // current epoch early in order to hold that validator transitions happen at epoch boundaries.
        if let (Active, Defined | Disabled | Jailed | Tombstoned) = (old_state, new_state) {
            self.set_end_epoch_flag();
        }

        match (old_state, new_state) {
            (Defined, Inactive) => {
                // The validator has reached the minimum threshold to be indexed by
                // the staking component.
                tracing::debug!(identity_key = ?identity_key, "validator has reached minimum stake threshold to be considered inactive");
                self.add_consensus_set_index(identity_key);
                self.put(validator_state_path, Inactive);
            }
            (Inactive, Defined) => {
                // The validator has fallen below the minimum threshold to be
                // part of the "greater" consensus set.
                tracing::debug!(identity_key = ?identity_key, "validator has fallen below minimum stake threshold to be considered inactive");
                self.remove_consensus_set_index(identity_key);
                self.put(validator_state_path, Defined);
            }
            (Inactive, Active) => {
                let power = self.get_validator_power(identity_key).await;
                tracing::debug!(validator_identity = %identity_key, voting_power = ?power, "validator has become active");

                // The validators with the most voting power are selected to be part of the
                // chain's "Active set".
                self.set_validator_bonding_state(identity_key, Bonded);

                // Track the validator uptime, overwriting any prior tracking data.
                self.set_validator_uptime(
                    identity_key,
                    Uptime::new(
                        self.get_block_height().await?,
                        self.signed_blocks_window_len().await? as usize,
                    ),
                );

                // Finally, set the validator to be active.
                self.put(validator_state_path, Active);

                metrics::gauge!(metrics::MISSED_BLOCKS, "identity_key" => identity_key.to_string())
                    .increment(0.0);
            }

            (Active, new_state @ (Inactive | Disabled)) => {
                // When an active validator becomes inactive, or is disabled by its operator,
                // we need to start the unbonding process for its delegation pool. We keep it
                // in the consensus set, but it is no longer part of the "active set".
                tracing::debug!(validator_identity = %identity_key, "transitioning validator from active to inactive");

                // The validator's delegation pool begins unbonding.
                self.set_validator_bonding_state(
                    identity_key,
                    Unbonding {
                        unbonds_at_epoch: self
                            .compute_unbonding_epoch_for_validator(identity_key)
                            .await?,
                    },
                );

                metrics::gauge!(metrics::MISSED_BLOCKS, "identity_key" => identity_key.to_string())
                    .increment(0.0);

                self.put(validator_state_path, new_state);
            }
            (Jailed, Inactive) => {
                // After getting jailed, a validator can be released from jail when its operator
                // updates the validator definition. It is then considered inactive, unless its
                // delegation pool falls below the minimum threshold.

                // Here, we don't have anything to do, only allow the validator to return to society.
                tracing::debug!(validator_identity = %identity_key, "releasing validator from jail");
                self.put(validator_state_path, Inactive);
            }
            (Disabled, Inactive) => {
                // The validator was disabled by its operator, and was re-enabled. Since its
                // delegation pool was sufficiently large, it is considered inactive.
                tracing::debug!(validator_identity = %identity_key, "disabled validator has become inactive");
                self.put(validator_state_path, Inactive);
            }
            (Inactive | Jailed, Disabled) => {
                // The validator was disabled by its operator.

                // We record that the validator was disabled, so delegations to it are not processed.
                tracing::debug!(validator_identity = %identity_key, validator_state = ?old_state, "validator has been disabled");
                self.put(validator_state_path, Disabled);
            }
            (Active, Jailed) => {
                // An active validator has committed misbehavior (e.g. failing to sign a block),
                // we must punish it by applying a penalty to its delegation pool and start the
                // unbonding process. We also record that the validator was jailed, so delegations
                // to it are not processed.
                let penalty = self.get_stake_params().await?.slashing_penalty_downtime;

                // Record the slashing penalty on this validator.
                self.record_slashing_penalty(identity_key, Penalty::from_bps_squared(penalty))
                    .await;

                // The validator's delegation pool begins unbonding.  Jailed
                // validators are not unbonded immediately, because they need to
                // be held accountable for byzantine behavior for the entire
                // unbonding period.
                self.set_validator_bonding_state(
                    identity_key,
                    Unbonding {
                        unbonds_at_epoch: self
                            .compute_unbonding_epoch_for_validator(identity_key)
                            .await?,
                    },
                );

                // Finally, set the validator to be jailed.
                self.put(validator_state_path, Jailed);
            }
            (Active, Defined) => {
                // The validator was part of the active set, but its delegation pool fell below
                // the minimum threshold. We remove it from the active set and the consensus set.
                tracing::debug!(validator_identity = %identity_key, "validator has fallen below minimum stake threshold to be considered active");

                // The validator's delegation pool begins unbonding.
                self.set_validator_bonding_state(
                    identity_key,
                    Unbonding {
                        unbonds_at_epoch: self
                            .compute_unbonding_epoch_for_validator(identity_key)
                            .await?,
                    },
                );
                self.remove_consensus_set_index(identity_key);
                self.put(validator_state_path, Defined);
            }
            (Defined | Disabled | Inactive | Active | Jailed, Tombstoned) => {
                // We have processed evidence of byzantine behavior for this validator.
                // It must be terminated and its delegation pool is slashed with a high
                // penalty. We immediately unbond the validator's delegation pool, and
                // it is removed from the consensus set.
                let misbehavior_penalty =
                    self.get_stake_params().await?.slashing_penalty_misbehavior;

                // Record the slashing penalty on this validator.
                self.record_slashing_penalty(
                    identity_key,
                    Penalty::from_bps_squared(misbehavior_penalty),
                )
                .await;

                // Regardless of its current bonding state, the validator's
                // delegation pool is unbonded immediately, because the
                // validator has already had the maximum slashing penalty
                // applied.
                self.set_validator_bonding_state(identity_key, Unbonded);

                // Remove the validator from the consensus set.
                self.remove_consensus_set_index(identity_key);

                // Finally, set the validator to be tombstoned.
                self.put(validator_state_path, Tombstoned);
            }
            (Tombstoned, Tombstoned) => {
                tracing::debug!(validator_identity = %identity_key, "validator is already tombstoned");
                // See discussion in https://github.com/penumbra-zone/penumbra/pull/3761 for context.
                // The abridged summary is that applying a misbehavior penalty enough and immediately
                // unbonding the validator's delegation pool should be enough to deter misbehavior.
                // Considering every single misbehavior actions as "counts" that accumulate runs the
                // risk of cratering misconfigured validator into oblivion.
            }
            (Disabled, Defined) => {
                self.put(validator_state_path, Defined);
            }
            (Defined, Disabled) => {
                self.put(validator_state_path, Disabled);
            }

            /* Bad transitions */
            (Defined | Jailed | Disabled, Active) => {
                anyhow::bail!(
                    "only Inactive validators can become Active: identity_key={}, old_state={:?}",
                    identity_key,
                    old_state
                )
            }
            (Jailed, Defined) => {
                anyhow::bail!(
                    "only inactive validators can become defined: identity_key={}, old_state={:?}",
                    identity_key,
                    old_state
                )
            }
            (Inactive | Disabled | Defined, Jailed) => anyhow::bail!(
                "only active validators can get jailed: state={:?}, identity_key={}",
                old_state,
                identity_key
            ),
            (Tombstoned, Defined | Disabled | Inactive | Active | Jailed) => {
                anyhow::bail!(
                    "tombstoning is permanent, identity_key={}, next_state={:?}",
                    identity_key,
                    new_state
                )
            }

            /* Identities: no-ops */
            (Defined, Defined) => { /* no-op */ }
            (Inactive, Inactive) => { /* no-op */ }
            (Active, Active) => { /* no-op */ }
            (Jailed, Jailed) => { /* no-op */ }
            (Disabled, Disabled) => { /* no-op */ }
        }

        // Update the validator metrics once the state transition has been applied.
        match old_state {
            Defined => metrics::gauge!(metrics::DEFINED_VALIDATORS).decrement(1.0),
            Inactive => metrics::gauge!(metrics::INACTIVE_VALIDATORS).decrement(1.0),
            Active => metrics::gauge!(metrics::ACTIVE_VALIDATORS).decrement(1.0),
            Disabled => metrics::gauge!(metrics::DISABLED_VALIDATORS).decrement(1.0),
            Jailed => metrics::gauge!(metrics::JAILED_VALIDATORS).decrement(1.0),
            Tombstoned => metrics::gauge!(metrics::TOMBSTONED_VALIDATORS).decrement(1.0),
        };
        match new_state {
            Defined => metrics::gauge!(metrics::DEFINED_VALIDATORS).increment(1.0),
            Inactive => metrics::gauge!(metrics::INACTIVE_VALIDATORS).increment(1.0),
            Active => metrics::gauge!(metrics::ACTIVE_VALIDATORS).increment(1.0),
            Disabled => metrics::gauge!(metrics::DISABLED_VALIDATORS).increment(1.0),
            Jailed => metrics::gauge!(metrics::JAILED_VALIDATORS).increment(1.0),
            Tombstoned => metrics::gauge!(metrics::TOMBSTONED_VALIDATORS).increment(1.0),
        };

        // Emit an event indicating the state change:
        self.record_proto(event::validator_state_change(
            identity_key,
            Some(old_state),
            new_state,
        ));

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
        let initial_rate_data = RateData {
            identity_key: validator.identity_key.clone(),
            epoch_index: genesis_base_rate.epoch_index,
            validator_reward_rate: 0u128.into(),
            validator_exchange_rate: 1_0000_0000u128.into(), // 1 represented as 1e8
        };

        // The initial allocations to the validator are specified in `genesis_allocations`.
        // We use these to determine the initial voting power for each validator.
        let delegation_id = DelegationToken::from(validator.identity_key.clone()).id();
        let total_delegation_tokens = genesis_allocations
            .get(&delegation_id)
            .copied()
            .unwrap_or(0u64.into());
        let power = initial_rate_data.voting_power(total_delegation_tokens);

        self.add_validator_inner(
            validator.clone(),
            initial_rate_data,
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

    /// Add a validator after genesis, which will start in a [`validator::State::Defined`]
    /// state with zero voting power, and unbonded delegation tokens. This is the default
    /// "initial" state for a validator.
    async fn add_validator(&mut self, validator: Validator, rate_data: RateData) -> Result<()> {
        // We don't immediately report the validator voting power to CometBFT
        // until it becomes active.
        self.add_validator_inner(
            validator.clone(),
            rate_data,
            validator::State::Defined,
            validator::BondingState::Unbonded,
            0u128.into(),
        )
        .await
    }

    /// Record a new validator definition and prime its initial state.
    /// This method is used for both genesis and post-genesis validators.
    /// In the former case, the validator starts in `[validator::State::Active]`
    /// state, while in the latter case, it starts in `[validator::State::Defined]`.
    ///
    /// # Errors
    /// This method errors if the initial state is not one of the two valid
    /// initial states. Or if the voting power is negative.
    async fn add_validator_inner(
        &mut self,
        validator: Validator,
        initial_rate_data: RateData,
        initial_state: validator::State,
        initial_bonding_state: validator::BondingState,
        initial_voting_power: Amount,
    ) -> Result<()> {
        tracing::debug!(validator_definition = ?validator, ?initial_state, ?initial_bonding_state, ?initial_voting_power, ?initial_rate_data, "adding validator");
        if !matches!(initial_state, State::Defined | State::Active) {
            anyhow::bail!(
                "validator (identity_key={}) cannot have initial_state={:?}",
                validator.identity_key,
                initial_state
            )
        }
        // TODO(erwan): add more guards for voting power and nonsensical initial states.
        // in a separate PR, will move this up closer to `add_validator` - i don't want to
        // clutter the diff for now.
        let id = validator.identity_key.clone();

        // First, we record the validator definition in the general validator index:
        self.put(
            state_key::validators::definitions::by_id(&id),
            validator.clone(),
        );
        // Then, we create a mapping from the validator's consensus key to its
        // identity key, so we can look up the validator by its consensus key, and
        // vice-versa.
        self.register_consensus_key(&validator.identity_key, &validator.consensus_key)
            .await;
        // We register the validator's delegation token in the token registry...
        self.register_denom(&DelegationToken::from(&id).denom())
            .await?;
        // ... and its reward rate data in the JMT.
        self.set_validator_rate_data(&id, initial_rate_data);

        // We initialize the validator's state, power, and bonding state.
        self.set_initial_validator_state(&id, initial_state)?;
        self.set_validator_power(&id, initial_voting_power)?;
        self.set_validator_bonding_state(&id, initial_bonding_state);

        // For genesis validators, we also need to add them to the consensus set index.
        if initial_state == validator::State::Active {
            self.add_consensus_set_index(&id);
        }

        // Finally, update metrics for the new validator.
        match initial_state {
            validator::State::Active => {
                metrics::gauge!(metrics::ACTIVE_VALIDATORS).increment(1.0);
            }
            validator::State::Defined => {
                metrics::gauge!(metrics::DEFINED_VALIDATORS).increment(1.0);
            }
            _ => unreachable!("the initial state was validated by the guard condition"),
        };

        metrics::gauge!(metrics::MISSED_BLOCKS, "identity_key" => id.to_string()).increment(0.0);

        Ok(())
    }

    /// Update a validator definition
    #[tracing::instrument(skip(self, validator), fields(id = ?validator.identity_key))]
    async fn update_validator(&mut self, validator: Validator) -> Result<()> {
        use validator::State::*;

        tracing::debug!(definition = ?validator, "updating validator definition");
        let id = &validator.identity_key;
        let current_state = self
            .get_validator_state(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("updated validator not found in JMT"))?;

        tracing::debug!(?current_state, ?validator.enabled, "updating validator state");

        match (current_state, validator.enabled) {
            (Active | Inactive | Jailed | Defined | Disabled, false) => {
                // The operator has disabled their validator.
                self.set_validator_state(id, Disabled).await?;
            }
            (Disabled, true) => {
                // The operator has re-enabled their validator, if it has enough stake it will become
                // inactive, otherwise it will become defined.
                let min_validator_stake = self.get_stake_params().await?.min_validator_stake;
                let current_validator_rate = self
                    .get_validator_rate(id)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("updated validator not found in JMT"))?;
                let delegation_token_supply = self
                    .token_supply(&DelegationToken::from(id).id())
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("updated validator not found in JMT"))?;
                let unbonded_amount =
                    current_validator_rate.unbonded_amount(delegation_token_supply);

                if unbonded_amount >= min_validator_stake {
                    self.set_validator_state(id, Inactive).await?;
                } else {
                    self.set_validator_state(id, Defined).await?;
                }
            }
            (Jailed, true) => {
                // Treat updates to jailed validators as unjail requests.
                // If the validator has enough stake, it will become inactive, otherwise it will become defined.
                let min_validator_stake = self.get_stake_params().await?.min_validator_stake;
                let validator_rate_data = self
                    .get_validator_rate(id)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("updated validator not found in JMT"))?;
                let delegation_pool_size = self
                    .token_supply(&DelegationToken::from(id).id())
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("updated validator not found in JMT"))?;

                let unbonded_pool_size = validator_rate_data.unbonded_amount(delegation_pool_size);

                if unbonded_pool_size.value() >= min_validator_stake.value() {
                    self.set_validator_state(id, Inactive).await?;
                } else {
                    self.set_validator_state(id, Defined).await?;
                }
            }
            (Active | Inactive, true) => {
                // This validator update does not affect the validator's state.
            }
            (Defined, true) => {
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

        self.put(state_key::validators::definitions::by_id(id), validator);

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
        let mut validator_identity_stream = self.consensus_set_stream()?;
        while let Some(identity_key) = validator_identity_stream.next().await {
            let identity_key = identity_key?;
            let state = self.get_validator_state(&identity_key);
            let uptime = self.get_validator_uptime(&identity_key);
            let consensus_key = self.fetch_validator_consensus_key(&identity_key);
            js.spawn(async move {
                let state = state
                    .await?
                    .expect("every known validator must have a recorded state");

                match state {
                    validator::State::Active => {
                        // If the validator is active, we need its consensus key and current uptime data:
                        Ok(Some((
                            identity_key,
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
                metrics::gauge!(metrics::MISSED_BLOCKS, "identity_key" => identity_key.to_string())
                    .increment(uptime.num_missed_blocks() as f64);

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

    /// Process evidence of byzantine behavior from CometBFT.
    ///
    /// # Errors
    /// Returns an error if the validator is not found in the JMT.
    async fn process_evidence(&mut self, evidence: &Misbehavior) -> Result<()> {
        let validator = self
            .get_validator_by_cometbft_address(&evidence.validator.address)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "attempted to slash unknown validator with evidence={:?}",
                    evidence
                )
            })?;

        self.set_validator_state(&validator.identity_key, validator::State::Tombstoned)
            .await
    }
}

impl<T: StateWrite + ?Sized> ValidatorManager for T {}
