use {
    crate::{
        component::{
            metrics,
            stake::{ConsensusIndexWrite, RateDataWrite},
            validator_handler::{
                validator_store::ValidatorPoolTracker, ValidatorDataRead, ValidatorDataWrite,
            },
            StateReadExt as _, StateWriteExt as _,
        },
        event,
        rate::{BaseRateData, RateData},
        state_key,
        validator::{
            self,
            BondingState::*,
            State::{self, *},
            Validator,
        },
        DelegationToken, IdentityKey, Penalty, Uptime,
    },
    anyhow::{ensure, Result},
    async_trait::async_trait,
    cnidarium::StateWrite,
    penumbra_sdk_asset::asset,
    penumbra_sdk_num::Amount,
    penumbra_sdk_proto::{DomainType as _, StateWriteProto},
    penumbra_sdk_sct::component::{
        clock::{EpochManager, EpochRead},
        StateReadExt as _,
    },
    penumbra_sdk_shielded_pool::component::AssetRegistry,
    std::collections::BTreeMap,
    tendermint::abci::types::Misbehavior,
    tracing::{instrument, Instrument},
};

#[async_trait]
/// Defines the validator state machine of the staking component.
///
/// # Overview
/// An interface to the validator state machine.
///
/// ## Validator management
/// - Add validator definition via [`add_validator`].
/// - Update validator definitions via [`update_validator_definition`].
/// - Process byzantine behavior evidence via [`process_evidence`].
///
/// ## State machine interface
/// - A fallible state transition function via [`set_validator_state`].
/// - A safer handle to tentatively explore state transitions via [`try_precursor_transition`].
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
/// [`process_evidence`]: Self::process_evidence
pub trait ValidatorManager: StateWrite {
    /// Execute a legal state transition, updating the validator records and
    /// implementing the necessary side effects.
    ///
    /// Returns a `(old_state, new_state)` tuple, corresponding to the executed transition.
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
    ) -> Result<(State, State)> {
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

    // Inner function pretends to be the outer one, so we can include `old_state`
    // in the tracing span.  This way, we don't need to include any information
    // in tracing events inside the function about what the state transition is,
    // because it's already attached to the span.
    #[instrument(skip(self), name = "set_validator_state")]
    async fn set_validator_state_inner(
        &mut self,
        identity_key: &IdentityKey,
        old_state: validator::State,
        new_state: validator::State,
    ) -> Result<(State, State)> {
        let validator_state_path = state_key::validators::state::by_id(identity_key);

        let current_height = self.get_block_height().await?;

        // Using the start height of the current epoch let us do block based unbonding delays without
        // requiring to bind actions to a specific block height (instead they bind to a whole epoch).
        let unbonding_start_height = self.get_epoch_by_height(current_height).await?.start_height;

        tracing::debug!("trying to execute a state transition");

        // Validator state transitions are usually triggered by an epoch transition. The exception
        // to this rule is when a validator exits the active set. In this case, we want to end the
        // current epoch early in order to hold that validator transitions happen at epoch boundaries.
        if let (Active, Defined | Disabled | Jailed | Tombstoned) = (old_state, new_state) {
            tracing::info!("signaling early epoch end as a result of validator state transition");
            self.set_end_epoch_flag();
        }

        // Determine if the state transition is valid, returning an error otherwise.
        match (old_state, new_state) {
            (Defined | Disabled | Jailed, Inactive) => {
                // The validator has enough stake to be considered for the consensus set.
                self.add_consensus_set_index(identity_key);
            }
            (Inactive | Jailed | Disabled, Defined) => {
                // The validator's delegation pool has fallen below the `min_validator_stake` threshold.
                // If necessary, the epoch-handler will deindex this validator after processing it.
            }
            (Inactive | Jailed | Defined, Disabled) => {
                // The validator was disabled by its operator.
                // If necessary, the epoch-handler will deindex this validator after processing it.
                // We record the height at which the validator was disabled outside of the `match`.
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
                // In the special case of a validator being disabled, we record the height at which it was disabled.
                self.set_validator_bonding_state(
                    identity_key,
                    Unbonding {
                        unbonds_at_height: self
                            .compute_unbonding_height(identity_key, unbonding_start_height)
                            .await?
                            .expect("active validators MUST be bonded"),
                    },
                );
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

                // The validator's delegation pool begins unbonding.  Jailed
                // validators are not unbonded immediately, because they need to
                // be held accountable for byzantine behavior for the entire
                // unbonding period.
                let unbonds_at_height = self
                    .compute_unbonding_height(identity_key, unbonding_start_height)
                    .await?
                    .expect("active validators MUST be bonded");

                self.set_validator_bonding_state(identity_key, Unbonding { unbonds_at_height });

                tracing::debug!(penalty, unbonds_at_height, "jailed validator");
            }

            (Defined | Disabled | Inactive | Active | Jailed, Tombstoned) => {
                // When we detect byzantine misbehavior from a validator, we:
                // 1. Record the maximum slashing penalty for the corresponding pool
                // 2. Immediately unbond its delegation pool
                // 3. Forbid new delegations
                //
                // Later, during end-epoch processing, we remove the validator from the
                // consensus set index.
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
                    "validator has been tombstoned and slashed"
                );
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

        // Now that we have validated the state transition, we can record the last disabled height.
        // Doing this here lets us keep the match statement clean and focused on the critical transitions.
        if new_state == Disabled {
            self.set_last_disabled_height(identity_key, current_height)
        }

        // At this point, we are guaranteed that this state transition is valid.
        tracing::info!("successful state transition");
        self.put(validator_state_path, new_state);

        self.record_proto(
            event::EventValidatorStateChange {
                identity_key: *identity_key,
                state: new_state,
            }
            .to_proto(),
        );

        Ok((old_state, new_state))
    }

    #[instrument(skip(self))]
    /// Try to perform a state transition in/out of the `Defined` precursor state.
    /// If successful, returns the new state, otherwise returns `None`.
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

    /// Add a new genesis validator starting in the [`Active`] state with its
    /// genesis allocation entirely bonded.
    #[instrument(skip(self, genesis_allocations))]
    async fn add_genesis_validator(
        &mut self,
        genesis_allocations: &BTreeMap<asset::Id, Amount>,
        genesis_base_rate: &BaseRateData,
        validator: Validator,
    ) -> Result<()> {
        let initial_validator_rate = RateData {
            identity_key: validator.identity_key.clone(),
            validator_reward_rate: genesis_base_rate.base_reward_rate.clone(),
            validator_exchange_rate: genesis_base_rate.base_exchange_rate.clone(),
        };
        // The initial allocations to the validator are specified in `genesis_allocations`.
        // In this case, the validator's delegation pool size is exactly its allocation
        // because we hardcoded the exchange rate to 1.
        let delegation_id = DelegationToken::from(validator.identity_key.clone()).id();
        let total_delegation_tokens = genesis_allocations
            .get(&delegation_id)
            .copied()
            .unwrap_or_else(Amount::zero);
        let power = initial_validator_rate.voting_power(total_delegation_tokens);

        tracing::debug!(?initial_validator_rate, ?power, "adding genesis validator");

        self.add_validator_inner(
            validator.clone(),
            initial_validator_rate,
            // All genesis validators start in the "Active" state:
            validator::State::Active,
            // All genesis validators start in the "Bonded" bonding state:
            validator::BondingState::Bonded,
            power,
            total_delegation_tokens,
        )
        .await?;

        // Here, we are in the special case of genesis validators. Since they start in
        // the active state we need to bundle the effects of the `Inactive -> Active`
        // state transition:
        // - add them to the consensus set index
        // - track their uptime
        self.add_consensus_set_index(&validator.identity_key);
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
            Amount::zero(),
            Amount::zero(),
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
    #[instrument(skip(self))]
    async fn add_validator_inner(
        &mut self,
        validator: Validator,
        initial_rate_data: RateData,
        initial_state: validator::State,
        initial_bonding_state: validator::BondingState,
        initial_voting_power: Amount,
        initial_delegation_pool_size: Amount,
    ) -> Result<()> {
        tracing::debug!("adding validator");
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
        self.register_consensus_key(&validator_identity, &validator.consensus_key);
        // We register the validator's delegation token in the token registry...
        self.register_denom(&DelegationToken::from(&validator_identity).denom())
            .await;
        // ... and its reward rate data in the JMT.
        self.set_validator_rate_data(&validator_identity, initial_rate_data);

        // Track the validator's definition in an event (the rest of the attributes will be tracked
        // in events emitted by the calls to set_* methods below).
        self.record_proto(
            event::EventValidatorDefinitionUpload {
                validator: validator.clone(),
            }
            .to_proto(),
        );

        // We initialize the validator's state, power, and bonding state.
        self.set_initial_validator_state(&validator_identity, initial_state)?;
        self.set_validator_power(&validator_identity, initial_voting_power)?;
        self.set_validator_bonding_state(&validator_identity, initial_bonding_state);
        self.set_validator_pool_size(&validator_identity, initial_delegation_pool_size);

        metrics::gauge!(metrics::MISSED_BLOCKS, "identity_key" => validator_identity.to_string())
            .increment(0.0);

        Ok(())
    }

    /// Create a new validator definition or update an existing one.
    /// # Errors
    /// This method errors if the validator state is not found in the state,
    /// or if the validator definition has been disabled too recently.
    #[tracing::instrument(skip(self, validator), fields(id = ?validator.identity_key))]
    async fn update_validator_definition(&mut self, validator: Validator) -> Result<()> {
        tracing::debug!(definition = ?validator, "updating validator definition");
        let id = &validator.identity_key;
        let current_state = self
            .get_validator_state(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("updated validator has no recorded state"))?;

        tracing::debug!(?current_state, ?validator.enabled, "updating validator state");

        match (current_state, validator.enabled) {
            (Active | Inactive | Jailed | Defined | Disabled, false) => {
                // The operator has disabled their validator.
                self.set_validator_state(id, Disabled).await?;
            }
            (Disabled, true) => {
                let last_disabled_height = self.get_last_disabled_height(id).await;
                if let Some(last_disabled) = last_disabled_height {
                    let current_height = self.get_block_height().await?;
                    let epoch_duration = self.get_sct_params().await?.epoch_duration;

                    // The actual delay is not too load-bearing, what we want here is to make sure that
                    // there is a buffer between the last disabled height and the current height.
                    // See #4067 for details about epoch-grinding.
                    let allowed_enabled_height = last_disabled.saturating_add(epoch_duration);
                    let wait_duration = current_height.saturating_sub(allowed_enabled_height);
                    ensure!(
                        current_height >= allowed_enabled_height,
                        "validator has been disabled too recently (last_disabled={}, current_height={}, epoch_duration={}), wait {} blocks",
                        last_disabled,
                        current_height,
                        epoch_duration,
                        wait_duration
                    );
                } else {
                    tracing::warn!(validator_identity = %id, "validator has no recorded last_disabled_height but is disabled");
                }
                // The operator has re-enabled their validator, if it has enough stake it will become
                // inactive, otherwise it will become defined.
                let min_validator_stake = self.get_stake_params().await?.min_validator_stake;
                let current_validator_rate =
                    self.get_validator_rate(id).await?.ok_or_else(|| {
                        anyhow::anyhow!("updated validator has no recorded rate data")
                    })?;
                let delegation_token_supply = self
                    .get_validator_pool_size(id)
                    .await
                    .unwrap_or_else(Amount::zero);
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
                    .ok_or_else(|| anyhow::anyhow!("updated validator has no recorded state"))?;
                let delegation_pool_size = self
                    .get_validator_pool_size(id)
                    .await
                    .unwrap_or_else(Amount::zero);

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
        self.register_consensus_key(&validator.identity_key, &validator.consensus_key);

        self.put(
            state_key::validators::definitions::by_id(id),
            validator.clone(),
        );

        // Track the validator's definition in an event.
        self.record_proto(event::EventValidatorDefinitionUpload { validator }.to_proto());

        Ok(())
    }

    /// Update the validator pool's bonding state.
    #[instrument(skip(self))]
    async fn process_validator_pool_state(
        &mut self,
        validator_identity: &IdentityKey,
        from_height: u64,
    ) -> Result<()> {
        let pool_state = self.get_validator_bonding_state(validator_identity).await;

        // If the pool is already unbonded, this will return the current epoch.
        let allowed_unbonding_height = self
            .compute_unbonding_height(validator_identity, from_height)
            .await?
            .unwrap_or(from_height); // If the pool is unbonded, the unbonding height is the current height.

        tracing::debug!(
            ?pool_state,
            ?allowed_unbonding_height,
            "processing validator pool state"
        );

        if from_height >= allowed_unbonding_height {
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
            .get_validator_definition_by_cometbft_address(&evidence.validator.address)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "attempted to slash unknown validator with evidence={:?}",
                    evidence
                )
            })?;

        let (old_state, new_state) = self
            .set_validator_state(&validator.identity_key, validator::State::Tombstoned)
            .await?;

        if let (Inactive | Jailed | Active, Tombstoned) = (old_state, new_state) {
            let current_height = self.get_block_height().await?;
            self.record_proto(
                event::EventTombstoneValidator::from_evidence(
                    current_height,
                    validator.identity_key.clone(),
                    evidence,
                )
                .to_proto(),
            );
        }

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> ValidatorManager for T {}
