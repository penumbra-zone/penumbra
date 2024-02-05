use crate::component::metrics;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::{FutureExt, StreamExt, TryStreamExt};
use penumbra_asset::{asset, STAKING_TOKEN_ASSET_ID};
use penumbra_distributions::component::StateReadExt as _;
use penumbra_sct::{
    component::clock::{EpochManager, EpochRead},
    epoch::Epoch,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    future::Future,
    pin::Pin,
    str::FromStr,
};
use validator::BondingState::*;

use cnidarium::{StateRead, StateWrite};
use penumbra_num::{fixpoint::U128x128, Amount};
use penumbra_proto::{state::future::DomainFuture, StateReadProto, StateWriteProto};
use penumbra_shielded_pool::component::{SupplyRead, SupplyWrite};
use sha2::{Digest, Sha256};
use tendermint::validator::Update;
use tendermint::{
    abci::types::{CommitInfo, Misbehavior},
    block, PublicKey,
};
use tracing::{instrument, Instrument};

use crate::{
    component::StateWriteExt as _,
    state_key,
    validator::{self},
    IdentityKey, Penalty, StateReadExt as _, Uptime,
};

#[async_trait]
pub(crate) trait ValidatorManager: StateWrite {
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
        let old_state = self.validator_state(identity_key).await?.ok_or_else(|| {
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
        let state_key = state_key::state_by_validator(identity_key).to_owned();

        // Doing a single tuple match, rather than matching on substates,
        // ensures we exhaustively cover all possible state transitions.
        use validator::State::*;

        // Whenever a validator transitions out of the active state to a disabled, jailed, or
        // tombstoned state, this means that we need to explicitly signal the end of an epoch,
        // because there has been a change to the validator set outside of a normal epoch
        // transition. All other validator state transitions (including from active to inactive) are
        // triggered by epoch transitions themselves, or don't immediately affect the active
        // validator set.
        if let (Active, Defined | Disabled | Jailed | Tombstoned) = (old_state, new_state) {
            self.set_end_epoch_flag();
        }

        match (old_state, new_state) {
            (Defined, Inactive) => {
                // The validator has reached the minimum threshold to become
                // part of the "greater" consensus set. We index it and will
                // iterate over it during end-epoch processing.
                tracing::debug!(identity_key = ?identity_key, "validator has reached minimum stake threshold to be considered inactive");
                self.add_consensus_set_index(identity_key);
            }
            (Inactive, Defined) => {
                // The validator has fallen below the minimum threshold to be
                // part of the "greater" consensus set. We remove it from the
                // index.
                tracing::debug!(identity_key = ?identity_key, "validator has fallen below minimum stake threshold to be considered inactive");
                self.remove_consensus_set_index(identity_key);
            }
            (Inactive, Active) => {
                // When the validator's delegation pool is one of the largest one
                // of the chain, its status becomes promoted to `Active`. We also
                // say that it is part of the "active set".

                // We need to update its bonding state to `Bonded` and start tracking
                // its uptime/liveness.
                self.set_validator_bonding_state(identity_key, Bonded);

                // Track the validator uptime, overwriting any prior tracking data.
                self.set_validator_uptime(
                    identity_key,
                    Uptime::new(
                        self.get_block_height().await?,
                        self.signed_blocks_window_len().await? as usize,
                    ),
                );

                let power = self.validator_power(identity_key).await;

                // Finally, set the validator to be active.
                self.put(state_key, Active);

                metrics::gauge!(metrics::MISSED_BLOCKS, 0.0, "identity_key" => identity_key.to_string());

                tracing::debug!(validator_identity = %identity_key, voting_power = ?power, "validator has become active");
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

                self.put(state_key, new_state);

                metrics::gauge!(metrics::MISSED_BLOCKS, 0.0, "identity_key" => identity_key.to_string());
            }
            (Jailed, Inactive) => {
                // After getting jailed, a validator can be released from jail when its operator
                // updates the validator definition. It is then considered inactive, unless its
                // delegation pool falls below the minimum threshold.

                // Here, we don't have anything to do, only allow the validator to return to society.
                tracing::debug!(validator_identity = %identity_key, "releasing validator from jail");
                self.put(state_key, Inactive);
            }
            (Disabled, Inactive) => {
                // The validator was disabled by its operator, and was re-enabled. Since its
                // delegation pool was sufficiently large, it is considered inactive.
                tracing::debug!(validator_identity = %identity_key, "disabled validator has become inactive");
                self.put(state_key, Inactive);
            }
            (Inactive | Jailed, Disabled) => {
                // The validator was disabled by its operator.

                // We record that the validator was disabled, so delegations to it are not processed.
                tracing::debug!(validator_identity = %identity_key, validator_state = ?old_state, "validator has been disabled");
                self.put(state_key, Disabled);
            }
            (Active, Jailed) => {
                // An active validator has committed misbehavior (e.g. failing to sign a block),
                // we must punish it by applying a penalty to its delegation pool and start the
                // unbonding process. We also record that the validator was jailed, so delegations
                // to it are not processed.
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
                        unbonds_at_epoch: self
                            .compute_unbonding_epoch_for_validator(identity_key)
                            .await?,
                    },
                );

                // Finally, set the validator to be jailed.
                self.put(state_key, Jailed);
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
                self.put(state_key, Defined);
            }
            (Defined | Disabled | Inactive | Active | Jailed, Tombstoned) => {
                // We have processed evidence of byzantine behavior for this validator.
                // It must be terminated and its delegation pool is slashed with a high
                // penalty. We immediately unbond the validator's delegation pool, and
                // it is removed from the consensus set.
                let penalty = self.get_stake_params().await?.slashing_penalty_misbehavior;

                // Record the slashing penalty on this validator.
                self.record_slashing_penalty(identity_key, Penalty::from_bps_squared(penalty))
                    .await?;

                // Regardless of its current bonding state, the validator's
                // delegation pool is unbonded immediately, because the
                // validator has already had the maximum slashing penalty
                // applied.
                self.set_validator_bonding_state(identity_key, Unbonded);

                // Remove the validator from the consensus set.
                self.remove_consensus_set_index(identity_key);

                // Finally, set the validator to be tombstoned.
                self.put(state_key, Tombstoned);
            }
            /* ****** bad transitions ******* */
            (Defined | Jailed | Disabled, Active) => {
                anyhow::bail!(
                    "only Inactive validators can become Active: identity_key={}, state={:?}",
                    identity_key,
                    old_state
                )
            }
            (Jailed, Defined) => {
                anyhow::bail!(
                    "only inactive validators can become defined: identity_key={}, state={:?}",
                    identity_key,
                    old_state
                )
            }
            (Inactive | Jailed | Disabled | Defined, Jailed) => anyhow::bail!(
                "only active validators can get jailed: state={:?}, identity_key={}",
                old_state,
                identity_key
            ),
            (Tombstoned, Defined | Disabled | Inactive | Active | Jailed | Tombstoned) => {
                anyhow::bail!("tombstoning is permanent, identity_key={}", identity_key)
            }
            (Inactive, Inactive) => { /* no-op */ }
            (Disabled, Disabled) => { /* no-op */ }
            (Active, Active) => { /* no-op */ }
            (Defined, Defined) => { /* no-op */ }
            (Defined, Disabled) => { /* no-op */ }
            (Disabled, Defined) => { /* no-op */ }
        }

        // Update the validator metrics once the state transition has been applied.
        match old_state {
            Defined => metrics::decrement_gauge!(metrics::DEFINED_VALIDATORS, 1.0),
            Inactive => metrics::decrement_gauge!(metrics::INACTIVE_VALIDATORS, 1.0),
            Active => metrics::decrement_gauge!(metrics::ACTIVE_VALIDATORS, 1.0),
            Disabled => metrics::decrement_gauge!(metrics::DISABLED_VALIDATORS, 1.0),
            Jailed => metrics::decrement_gauge!(metrics::JAILED_VALIDATORS, 1.0),
            Tombstoned => metrics::decrement_gauge!(metrics::TOMBSTONED_VALIDATORS, 1.0),
        };
        match new_state {
            Defined => metrics::increment_gauge!(metrics::DEFINED_VALIDATORS, 1.0),
            Inactive => metrics::increment_gauge!(metrics::INACTIVE_VALIDATORS, 1.0),
            Active => metrics::increment_gauge!(metrics::ACTIVE_VALIDATORS, 1.0),
            Disabled => metrics::increment_gauge!(metrics::DISABLED_VALIDATORS, 1.0),
            Jailed => metrics::increment_gauge!(metrics::JAILED_VALIDATORS, 1.0),
            Tombstoned => metrics::increment_gauge!(metrics::TOMBSTONED_VALIDATORS, 1.0),
        };

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> ValidatorManager for T {}
