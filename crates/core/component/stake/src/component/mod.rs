pub mod action_handler;
pub mod metrics;
pub mod rpc;
pub mod validator_handler;

mod epoch_handler;
mod stake;

pub use stake::Staking;
// Max validator power is 1152921504606846975 (i64::MAX / 8)
// https://github.com/tendermint/tendermint/blob/master/types/validator_set.go#L25
pub const MAX_VOTING_POWER: u128 = 1152921504606846975;
pub const FP_SCALING_FACTOR: Lazy<U128x128> = Lazy::new(|| U128x128::from(1_0000_0000u128));

// pub use self::metrics::register_metrics;

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
    component::validator_handler::ValidatorDataRead,
    component::validator_handler::ValidatorManager,
    params::StakeParameters,
    rate::{BaseRateData, RateData},
    state_key,
    validator::{self, State, Validator},
    CurrentConsensusKeys, DelegationChanges, FundingStreams, Penalty, Uptime,
    {DelegationToken, IdentityKey},
};
use crate::{Delegate, Undelegate};
use once_cell::sync::Lazy;

pub trait ConsensusUpdateRead: StateRead {
    /// Returns a list of validator updates to send to Tendermint.
    ///
    /// Set during `end_block`.
    fn tendermint_validator_updates(&self) -> Option<Vec<Update>> {
        self.object_get(state_key::internal::tendermint_validator_updates())
            .unwrap_or(None)
    }
}

impl<T: StateRead + ?Sized> ConsensusUpdateRead for T {}

trait ConsensusUpdateWrite: StateWrite {
    fn put_tendermint_validator_updates(&mut self, updates: Vec<Update>) {
        tracing::debug!(?updates);
        self.object_put(
            state_key::internal::tendermint_validator_updates(),
            Some(updates),
        )
    }
}

impl<T: StateWrite + ?Sized> ConsensusUpdateWrite for T {}

/// Extension trait providing read access to staking data.
#[async_trait]
pub trait StateReadExt: StateRead {
    /// Gets the stake parameters from the JMT.
    async fn get_stake_params(&self) -> Result<StakeParameters> {
        self.get(state_key::stake_params())
            .await?
            .ok_or_else(|| anyhow!("Missing StakeParameters"))
    }

    /// Indicates if the stake parameters have been updated in this block.
    fn stake_params_updated(&self) -> bool {
        self.object_get::<()>(state_key::stake_params_updated())
            .is_some()
    }

    async fn signed_blocks_window_len(&self) -> Result<u64> {
        Ok(self.get_stake_params().await?.signed_blocks_window_len)
    }

    async fn missed_blocks_maximum(&self) -> Result<u64> {
        Ok(self.get_stake_params().await?.missed_blocks_maximum)
    }

    /// Delegation changes accumulated over the course of this block, to be
    /// persisted at the end of the block for processing at the end of the next
    /// epoch.
    fn get_delegation_changes_tally(&self) -> DelegationChanges {
        self.object_get(state_key::internal::delegation_changes())
            .unwrap_or_default()
    }

    async fn get_current_base_rate(&self) -> Result<BaseRateData> {
        self.get(state_key::current_base_rate())
            .await
            .map(|rate_data| rate_data.expect("rate data must be set after init_chain"))
    }

    fn get_previous_base_rate(&self) -> Option<BaseRateData> {
        self.object_get(state_key::previous_base_rate())
    }

    /// Returns the funding queue from object storage (end-epoch).
    fn get_funding_queue(&self) -> Option<Vec<(IdentityKey, FundingStreams, Amount)>> {
        self.object_get(state_key::validators::rewards::object_storage_key())
    }

    async fn get_delegation_changes(&self, height: block::Height) -> Result<DelegationChanges> {
        Ok(self
            .get(&state_key::delegation_changes_by_height(height.value()))
            .await?
            .ok_or_else(|| anyhow!("missing delegation changes for block {}", height))?)
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
    fn put_delegation_changes(&mut self, delegation_changes: DelegationChanges) {
        self.object_put(
            state_key::internal::delegation_changes(),
            delegation_changes,
        )
    }

    /// Push an entry in the delegation queue for the current block (object-storage).
    fn push_delegation(&mut self, delegation: Delegate) {
        let mut changes = self.get_delegation_changes_tally();
        changes.delegations.push(delegation);
        self.put_delegation_changes(changes);
    }

    /// Push an entry in the undelegation queue for the current block (object-storage).
    fn push_undelegation(&mut self, undelegation: Undelegate) {
        let mut changes = self.get_delegation_changes_tally();
        changes.undelegations.push(undelegation);
        self.put_delegation_changes(changes);
    }

    #[instrument(skip(self))]
    fn queue_staking_rewards(
        &mut self,
        staking_reward_queue: Vec<(IdentityKey, FundingStreams, Amount)>,
    ) {
        self.object_put(
            state_key::validators::rewards::object_storage_key(),
            staking_reward_queue,
        )
    }

    async fn register_consensus_key(
        &mut self,
        identity_key: &IdentityKey,
        consensus_key: &PublicKey,
    ) {
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
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

pub trait StakingDataInternalRead: StateRead {
    async fn get_penalty_in_epoch(
        &self,
        id: &IdentityKey,
        epoch_index: u64,
    ) -> Result<Option<Penalty>> {
        self.get(&state_key::penalty_in_epoch(id, epoch_index))
            .await
    }

    async fn get_penalty_for_range(&self, id: &IdentityKey, start: u64, end: u64) -> Vec<Penalty> {
        let prefix = state_key::penalty_in_epoch_prefix(id);
        let all_penalties: BTreeMap<String, Penalty> = self
            .prefix::<Penalty>(&prefix)
            .try_collect()
            .await
            .unwrap_or_default();
        let start_key = state_key::penalty_in_epoch(id, start);
        let end_key = state_key::penalty_in_epoch(id, end);
        all_penalties
            .range(start_key..end_key)
            .map(|(_k, v)| v.to_owned())
            .collect()
    }

    fn compute_compounded_penalty(penalties: Vec<Penalty>) -> Penalty {
        let mut compounded = Penalty::from_percent(0);
        penalties
            .into_iter()
            .fold(compounded, |acc, penalty| acc.compound(penalty))
    }

    /// Returns the compounded penalty for the given validator over the half-open range of epochs [start, end).
    async fn compounded_penalty_over_range(
        &self,
        id: &IdentityKey,
        start: u64,
        end: u64,
    ) -> Result<Penalty> {
        let range = self.get_penalty_for_range(id, start, end).await;
        let compounded_penalty = Self::compute_compounded_penalty(range);
        Ok(compounded_penalty)
    }

    /// Calculate the amount of stake that is delegated to the currently active validator set,
    /// denominated in the staking token.
    #[instrument(skip(self))]
    async fn total_active_stake(&self) -> Result<Amount> {
        let mut total_active_stake = Amount::zero();

        let mut validator_stream = self.consensus_set_stream()?;
        while let Some(validator_identity) = validator_stream.next().await {
            let validator_identity = validator_identity?;
            let validator_state = self
                .get_validator_state(&validator_identity)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("validator (identity_key={}) is in the consensus set index but its state was not found", validator_identity)
                })?;
            if validator_state != validator::State::Active {
                continue;
            }

            let delegation_token_supply = self
                .token_supply(&DelegationToken::from(validator_identity).id())
                .await?
                .expect("delegation token should be known");

            let validator_rate = self
                .get_validator_rate(&validator_identity)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("validator (identity_key={}) is in the consensus set index but its rate data was not found", validator_identity)
                })?;

            // Add the validator's unbonded amount to the total active stake
            total_active_stake = total_active_stake
                .checked_add(&validator_rate.unbonded_amount(delegation_token_supply))
                .ok_or_else(|| {
                    anyhow::anyhow!("total active stake overflowed `Amount` (128 bits)")
                })?;
        }

        Ok(total_active_stake.into())
    }
}

impl<T: StateRead + ?Sized> StakingDataInternalRead for T {}

#[async_trait]
pub trait RateDataWrite: StateWrite {
    #[instrument(skip(self))]
    fn set_base_rate(&mut self, rate_data: BaseRateData) {
        tracing::debug!("setting base rate");
        self.put(state_key::current_base_rate().to_owned(), rate_data);
    }

    #[instrument(skip(self))]
    fn set_prev_base_rate(&mut self, rate_data: BaseRateData) {
        self.object_put(state_key::previous_base_rate(), rate_data);
    }

    async fn record_slashing_penalty(
        &mut self,
        identity_key: &IdentityKey,
        slashing_penalty: Penalty,
    ) -> Result<()> {
        let current_epoch_index = self.get_current_epoch().await?.index;

        let current_penalty = self
            .get_penalty_in_epoch(identity_key, current_epoch_index)
            .await?
            .unwrap_or(Penalty::from_percent(0));

        let new_penalty = current_penalty.compound(slashing_penalty);

        self.put(
            state_key::penalty_in_epoch(identity_key, current_epoch_index),
            new_penalty,
        );

        Ok(())
    }

    async fn set_delegation_changes(&mut self, height: block::Height, changes: DelegationChanges) {
        self.put(
            state_key::delegation_changes_by_height(height.value()),
            changes,
        );
    }
}

impl<T: StateWrite + ?Sized> RateDataWrite for T {}

#[async_trait]
pub trait ConsensusIndexRead: StateRead {
    /// Returns a stream of [`IdentityKey`]s of validators that are currently in the consensus set.
    /// This only excludes validators that do not meet the minimum validator stake requirement
    /// (see [`StakeParameters::min_validator_stake`]).
    fn consensus_set_stream(
        &self,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<IdentityKey>> + Send + 'static>>> {
        Ok(self
            .nonverifiable_prefix_raw(
                state_key::validators::index::consensus_set::prefix().as_bytes(),
            )
            .map(|res| {
                res.map(|(_, raw_identity_key)| {
                    // TODO(erwan): is this an opportunity to extend the proto overlay?
                    let str_identity_key = std::str::from_utf8(raw_identity_key.as_slice())
                        .expect("state keys should only have valid identity keys");
                    IdentityKey::from_str(str_identity_key)
                        .expect("state keys should only have valid identity keys")
                })
            })
            .boxed())
    }
}

impl<T: StateRead + ?Sized> ConsensusIndexRead for T {}

#[async_trait]
pub trait ConsensusIndexWrite: StateWrite {
    /// Add a validator identity to the consensus set index.
    /// The consensus set index includes any validator that has a delegation pool that
    /// is greater than [`StakeParameters::min_validator_stake`].
    /// TODO(erwan): We should split this into an `ValidatorIndex` extension traits.
    fn add_consensus_set_index(&mut self, identity_key: &IdentityKey) {
        tracing::debug!(validator = %identity_key, "adding validator identity to consensus set index");
        self.nonverifiable_put_raw(
            state_key::validators::index::consensus_set::by_id(identity_key)
                .as_bytes()
                .to_vec(),
            identity_key.to_string().as_bytes().to_vec(),
        );
    }

    /// Remove a validator identity from the consensus set index.
    /// The consensus set index includes any validator that has a delegation pool that
    /// is greater than [`StakeParameters::min_validator_stake`].
    fn remove_consensus_set_index(&mut self, identity_key: &IdentityKey) {
        tracing::debug!(validator = %identity_key, "removing validator identity from consensus set index");
        self.nonverifiable_delete(
            state_key::validators::index::consensus_set::by_id(identity_key)
                .as_bytes()
                .to_vec(),
        );
    }
}

impl<T: StateWrite + ?Sized> ConsensusIndexWrite for T {}
