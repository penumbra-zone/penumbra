use std::collections::BTreeMap;

use crate::{
    component::{
        metrics, stake::ConsensusIndexRead, stake::ConsensusIndexWrite, stake::RateDataWrite,
        validator_handler::ValidatorDataWrite,
    },
    rate::{BaseRateData, RateData},
    validator::{State, Validator},
    DelegationToken,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt as _;
use penumbra_num::Amount;
use penumbra_sct::{
    component::clock::{EpochManager, EpochRead},
    epoch::Epoch,
};
use penumbra_shielded_pool::component::{SupplyRead as _, SupplyWrite};
use sha2::{Digest as _, Sha256};
use tendermint::abci::types::{CommitInfo, Misbehavior};
use tokio::task::JoinSet;
use validator::BondingState::*;
use validator::State::*;

use cnidarium::StateWrite;
use penumbra_proto::StateWriteProto;
use tracing::{instrument, Instrument};

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
/// Defines the validator state machine of the staking component.
///
/// # Overview
/// This trait offers an interface to the validator staking state machine.
///
/// ## Validator management
/// - Add validator definition via [`add_validator`].
/// - Update validator definitions via [`update_validator_definition`].
///
/// ## State machine interface
/// - A fallible state transition function via [`set_validator_state`].
/// - A safer handle to tentatively explore state transitions via [`try_precursor_transition`].
///
/// ## Validator-specific logic
/// - Tracking a validator's uptime via [`track_uptime`].
/// - Process byzantine behavior evidence via [`process_evidence`].
///
/// # State machine diagram:
/// ```plaintext
///             ┌───────────────────────────────────────────────────────┐
///             │                      ┌──────────────────────────────┐ │
///             ▼                      ▼                              │ │
///     ╔═══════════════╗       ┌────────────┐                        │ │
///  ┌─▶║    Defined    ║◀─────▶│  Disabled  │                        │ │
///  │  ╚═══════════════╝       └────────────┘                        │ │
///  │          │                      │                              │ │
///  │          │                      ▼                              │ │
///  │          │               ┏━━━━━━━━━━━━┓                        │ │
///  │          └──────────────▶┃            ┃                        │ │
///  │                          ┃ Tombstoned ┃◀────────────┐          │ │
///  │          ┌──────────────▶┃            ┃             │          │ │
///  │          │               ┗━━━━━━━━━━━━┛             │          │ │
///  │          │                      ▲                   │          │ │
///  │          │                      │           ┌──────────────┐   │ │
///  │   ┌─────────────┐        ┌────────────┐     │              │◀──┘ │
///  └──▶│   Jailed    │◀───────│   Active   │◀───▶│   Inactive   │     │
///      └─────────────┘        └────────────┘     │              │◀────┘
///             │                                  └──────────────┘      
///             │                                          ▲             
///             └──────────────────────────────────────────┘             
///                                                                      
///     ╔═════════════════╗
///     ║ starting state  ║
///     ╚═════════════════╝
///     ┏━━━━━━━━━━━━━━━━━┓
///     ┃ terminal state  ┃
///     ┗━━━━━━━━━━━━━━━━━┛         
/// ```
///
/// [`add_validator`]: Self::add_validator
/// [`update_validator_definition`]: Self::update_validator_definition
/// [`set_validator_state`]: Self::set_validator_state
/// [`try_precursor_transition`]: Self::try_precursor_transition
/// [`track_uptime`]: Self::track_uptime
/// [`process_evidence`]: Self::process_evidence
pub trait ValidatorManager: StateWrite {
    /// Execute a legal state transition, updating the validator records and
    /// implementing the necessary side effects.
    ///
    /// # Errors
    /// This method errors on illegal state transitions, but will otherwise try to do what
    /// you ask it to do. It is the caller's responsibility to ensure that the state transitions
    /// are legal and pertinent.
    ///
    /// An error can also happen if the state is corrupted or pushed into an incoherent mode
    /// in this case, we return an error but there is no way to recover from those.
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
        let validator_state_path = state_key::validators::state::by_id(identity_key);

        // We use the current epoch index to compute the unbonding epoch for the validator,
        // when necessary.
        let current_epoch = self.get_current_epoch().await?;

        tracing::debug!("tentatively executing a state transition");

        // Validator state transitions are usually triggered by an epoch transition. The exception
        // to this rule is when a validator exits the active set. In this case, we want to end the
        // current epoch early in order to hold that validator transitions happen at epoch boundaries.
        if let (Active, Defined | Disabled | Jailed | Tombstoned) = (old_state, new_state) {
            tracing::info!("signaling early epoch end as a result of validator state transition");
            self.set_end_epoch_flag();
        }

        // Determine if the state transition is valid, returning an error otherwise,
        // if valid we update the state key with the specified `new_state`.
        match (old_state, new_state) {
            (Defined, Inactive) => {
                // The validator has reached the minimum threshold to be indexed by
                // the staking component, so we add it:
                self.add_consensus_set_index(identity_key);
            }
            (Inactive, Defined) => {
                // The validator delegation pool is below the `min_stake_threshold` stake parameter.
                // It will be de-indexed from the consensus set by the epoch-handler.
            }
            (Inactive, Active) => {
                // The delegator has been promoted into the active set, we initialize its uptime tracker,
                // and bond its delegation pool.
                self.set_validator_bonding_state(identity_key, Bonded);

                // Track the validator uptime, overwriting any prior tracking data.
                self.set_validator_uptime(
                    identity_key,
                    Uptime::new(
                        self.get_block_height().await?,
                        self.signed_blocks_window_len().await? as usize,
                    ),
                );

                let power = self.get_validator_power(identity_key).await;
                tracing::debug!(voting_power = ?power, "validator pool: bonded, uptime: tracked, setting validator to active");
            }

            (Active, Inactive | Defined | Disabled) => {
                // When a validator is honorably discharged from the active set, we begin unbonding
                // its delegation pool. The epoch-handler will decide whether it wants to keep it in
                // the consensus set index or not.
                self.set_validator_bonding_state(
                    identity_key,
                    Unbonding {
                        unbonds_at_epoch: self
                            .compute_unbonding_epoch(identity_key, current_epoch.index)
                            .await?,
                    },
                );
            }
            (Jailed, Defined) => {
                // A jailed validator has been released from jail by its operator, but its
                // delegation pool has fallen below the `min_stake_threshold`.
                //
                // End-epoch handler is responsible for choosing when to deindex the validator.
            }
            (Jailed, Inactive) => {
                // We bring back this validator into the consensus set index.
                self.add_consensus_set_index(identity_key);
            }
            (Disabled, Inactive) => {
                // The validator was disabled by its operator, and was re-enabled. Since its
                // delegation pool was sufficiently large, it is considered inactive.
                self.add_consensus_set_index(identity_key);
            }
            (Inactive | Jailed, Disabled) => {
                // The validator was disabled by its operator.
                // The epoch-handler is responsible for removing this identity from the consensus set index.
            }
            (Active, Jailed) => {
                // An active validator has missed too many blocks, we penalize it by
                // slashing its delegation pool, forbid new delegations, and start
                // unbonding its delegated stake.  The epoch-handler is responsible
                // for removing this identity from the consensus set index.
                let penalty = self.get_stake_params().await?.slashing_penalty_downtime;

                // Record the slashing penalty on this validator.
                self.record_slashing_penalty(identity_key, Penalty::from_bps_squared(penalty))
                    .await;

                // The validator's delegation pool begins unbonding.
                let unbonds_at_epoch = self
                    .compute_unbonding_epoch(identity_key, current_epoch.index)
                    .await?;

                // Note: `Jailed` validators are not unbonded immediately, so that they
                // can be held accountable for byzantine behavior.
                self.set_validator_bonding_state(identity_key, Unbonding { unbonds_at_epoch });

                tracing::debug!(penalty, unbonds_at_epoch, "jailed validator");
            }
            (Defined | Disabled | Inactive | Active | Jailed, Tombstoned) => {
                // We have processed evidence of byzantine behavior for this validator.
                // It will be terminated, its delegation pool slashed and unbonded.
                // The epoch-handler is responsible for removing it from the consensus
                // set index.
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

                tracing::info!(
                    misbehavior_penalty,
                    "tombstoning validator and unbonding its pool"
                );
            }
            (Disabled, Defined) => {
                /* valid state transition, but there is nothing to do other than updating the state */
            }
            (Defined, Disabled) => {
                /* valid state transition, but there is nothing to do other than updating the state */
            }

            /* Identities: no-ops */
            (Tombstoned, Tombstoned) => {
                tracing::debug!("validator is already tombstoned");
                // See discussion in https://github.com/penumbra-zone/penumbra/pull/3761 for context.
                // The abridged summary is that applying a misbehavior penalty once and immediately
                // unbonding the validator's delegation pool should be enough to deter misbehavior.
                // Considering every single misbehavior actions as "counts" that accumulate runs the
                // risk of cratering misconfigured validator into oblivion.
            }
            (Defined, Defined) => { /* no-op */ }
            (Inactive, Inactive) => { /* no-op */ }
            (Active, Active) => { /* no-op */ }
            (Jailed, Jailed) => { /* no-op */ }
            (Disabled, Disabled) => { /* no-op */ }

            /* Bad transitions */
            (Disabled | Defined | Jailed, Active) => {
                anyhow::bail!(
                    "only inactive validators can become active (identity={}, old_state={:?}, new_state={:?})",
                    identity_key,
                    old_state,
                    new_state,
                )
            }
            (Disabled | Defined | Inactive, Jailed) => {
                anyhow::bail!(
                    "only active validators can get jailed (identity={}, old_state={:?}, new_state={:?})",
                    identity_key,
                    old_state,
                    new_state
                )
            }
            (Tombstoned, Defined | Disabled | Inactive | Active | Jailed) => {
                anyhow::bail!(
                    "tombstoning is permanent, identity_key={}, new_state={:?}",
                    identity_key,
                    new_state
                )
            }
        }

        // At this point, we are guaranteed that this state transition is valid.
        tracing::info!("successful state transition");
        self.put(validator_state_path, new_state);

        Self::state_machine_metrics(old_state, new_state);

        Ok(())
    }

    #[instrument(skip(self))]
    /// Try to implement a state transition in/out of the `Defined` precursor state.
    /// Returns the new state if successful, and `None` otherwise.
    async fn try_precursor_transition(
        &mut self,
        validator_id: &IdentityKey,
        previous_state: validator::State,
        next_rate: &RateData,
        delegation_token_supply: Amount,
    ) -> Option<State> {
        // Conspicuously missing from this list are `Jailed | Disabled` validators.
        // This is because their transition MUST be triggered by a manual validator upload.
        if !matches!(previous_state, Defined | Inactive | Active) {
            return None;
        }

        let min_stake = self
            .get_stake_params()
            .await
            .expect("staking parameters are always set")
            .min_validator_stake;

        // We convert the delegation pool into staking tokens so that we can decide whether
        // the validator meets the minimum stake threshold.
        let unbonded_pool = next_rate.unbonded_amount(delegation_token_supply);

        tracing::debug!(
            %validator_id,
            ?delegation_token_supply,
            ?unbonded_pool,
            next_validator_exchange_rate = ?next_rate.validator_exchange_rate,
            ?previous_state,
            ?min_stake,
            "computed validator unbonded pool to explore precursor transition"
        );

        let has_minimum_stake = unbonded_pool >= min_stake;

        // Refer yourself to the state machine diagram for the logic behind these transitions.
        // Note: this could be refactored, no doubt, but it's going to look ugly either way.
        let new_state = match previous_state {
            Defined if has_minimum_stake => Inactive,
            Defined if !has_minimum_stake => Defined,
            Active if has_minimum_stake => Active,
            Active if !has_minimum_stake => Defined,
            Inactive if has_minimum_stake => Inactive,
            Inactive if !has_minimum_stake => Defined,
            _ => unreachable!("the previous state was validated by the guard condition"),
        };

        if new_state != previous_state {
            let _ = self
                .set_validator_state(validator_id, new_state)
                .await
                .expect("this must be a valid transition because we guard the method");
        }

        Some(new_state)
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
            .unwrap_or_else(Amount::zero);
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

        // TODO(erwan): add more guards - maybe.
        // It's not totally clear we want guards here, because it's convenient
        // to be able to add a validator with a nonsensical initial state for testing purposes.
        // We will know once if we run with the testing framework. If we remove it, we'll have to
        // expand the `match` down below.
        if !matches!(initial_state, State::Defined | State::Active) {
            anyhow::bail!(
                "validator (identity_key={}) cannot have initial_state={:?}",
                validator.identity_key,
                initial_state
            )
        }
        let validator_identity = validator.identity_key.clone();

        // First, we record the validator definition in the general validator index:
        self.put(
            state_key::validators::definitions::by_id(&validator_identity),
            validator.clone(),
        );
        // Then, we create a mapping from the validator's consensus key to its
        // identity key, so we can look up the validator by its consensus key, and
        // vice-versa.
        self.register_consensus_key(&validator_identity, &validator.consensus_key)
            .await;
        // We register the validator's delegation token in the token registry...
        self.register_denom(&DelegationToken::from(&validator_identity).denom())
            .await?;
        // ... and its reward rate data in the JMT.
        self.set_validator_rate_data(&validator_identity, initial_rate_data);

        // We initialize the validator's state, power, and bonding state.
        self.set_initial_validator_state(&validator_identity, initial_state)?;
        self.set_validator_power(&validator_identity, initial_voting_power)?;
        self.set_validator_bonding_state(&validator_identity, initial_bonding_state);

        // For genesis validators, we also need to add them to the consensus set index.
        if initial_state == validator::State::Active {
            self.add_consensus_set_index(&validator_identity);
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

        metrics::gauge!(metrics::MISSED_BLOCKS, "identity_key" => validator_identity.to_string())
            .increment(0.0);

        Ok(())
    }

    /// Update a validator definition
    #[tracing::instrument(skip(self, validator), fields(id = ?validator.identity_key))]
    async fn update_validator_definition(&mut self, validator: Validator) -> Result<()> {
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

                if unbonded_pool_size >= min_validator_stake {
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

    /// Update the validator pool's bonding state.
    #[instrument(skip(self))]
    async fn process_validator_pool_state(
        &mut self,
        validator_identity: &IdentityKey,
        at_epoch: Epoch,
    ) -> Result<()> {
        let pool_state = self.get_validator_bonding_state(validator_identity).await;

        // If the pool is already unbonded, this will return the current epoch.
        let unbonding_epoch_target = self
            .compute_unbonding_epoch(validator_identity, at_epoch.index)
            .await?;

        tracing::debug!(
            validator_identity = %validator_identity,
            ?pool_state,
            ?unbonding_epoch_target,
            "processing validator pool state");

        if at_epoch.index >= unbonding_epoch_target {
            // The validator's delegation pool has finished unbonding, so we
            // transition it to the Unbonded state.
            let _ = self
                .set_validator_bonding_state(validator_identity, Unbonded)
                .instrument(tracing::debug_span!(
                    "validator_pool_unbonded",
                    ?validator_identity
                ));
        }

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
    /// Evidence *MUST* be processed before `end_block` is called, because
    /// the evidence may trigger a validator state transition requiring
    /// an early epoch change.
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

    fn state_machine_metrics(old_state: validator::State, new_state: validator::State) {
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
    }
}

impl<T: StateWrite + ?Sized> ValidatorManager for T {}
