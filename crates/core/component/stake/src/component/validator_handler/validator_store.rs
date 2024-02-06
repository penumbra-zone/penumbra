use std::pin::Pin;

use crate::{
    rate::RateData,
    validator::{State, Validator},
};
use anyhow::Result;
use async_trait::async_trait;
use futures::{Future, FutureExt, TryStreamExt};
use penumbra_num::Amount;
use penumbra_sct::{component::clock::EpochRead, epoch::Epoch};
use tendermint::PublicKey;
use validator::BondingState::*;

use cnidarium::{StateRead, StateWrite};
use penumbra_proto::{state::future::DomainFuture, StateReadProto, StateWriteProto};
use tracing::instrument;

use crate::component::MAX_VOTING_POWER;
use crate::{
    component::StateReadExt as _,
    state_key,
    validator::{self},
    IdentityKey, Uptime,
};
#[async_trait]
pub trait ValidatorDataRead: StateRead {
    async fn get_validator_info(
        &self,
        identity_key: &IdentityKey,
    ) -> Result<Option<validator::Info>> {
        let validator = self.get_validator_definition(identity_key).await?;
        let status = self.get_validator_status(identity_key).await?;
        let rate_data = self.get_validator_rate(identity_key).await?;

        match (validator, status, rate_data) {
            (Some(validator), Some(status), Some(rate_data)) => Ok(Some(validator::Info {
                validator,
                status,
                rate_data,
            })),
            _ => Ok(None),
        }
    }

    fn get_validator_state(
        &self,
        identity_key: &IdentityKey,
    ) -> DomainFuture<validator::State, Self::GetRawFut> {
        self.get(&state_key::state_by_validator(identity_key))
    }

    async fn get_validator_bonding_state(
        &self,
        identity_key: &IdentityKey,
    ) -> Result<Option<validator::BondingState>> {
        self.get(&state_key::bonding_state_by_validator(identity_key))
            .await
    }

    /// Convenience method to assemble a [`ValidatorStatus`].
    async fn get_validator_status(
        &self,
        identity_key: &IdentityKey,
    ) -> Result<Option<validator::Status>> {
        let bonding_state = self.get_validator_bonding_state(identity_key).await?;
        let state = self.get_validator_state(identity_key).await?;
        let power = self.get_validator_power(identity_key).await?;
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

    fn get_validator_rate(
        &self,
        identity_key: &IdentityKey,
    ) -> Pin<Box<dyn Future<Output = Result<Option<RateData>>> + Send + 'static>> {
        self.get(&state_key::current_rate_by_validator(identity_key))
            .boxed()
    }

    fn get_validator_power(
        &self,
        validator: &IdentityKey,
    ) -> DomainFuture<Amount, Self::GetRawFut> {
        self.get(&state_key::power_by_validator(validator))
    }

    async fn get_validator_definition(
        &self,
        identity_key: &IdentityKey,
    ) -> Result<Option<Validator>> {
        self.get(&state_key::validators::definitions::by_id(identity_key))
            .await
    }

    fn get_validator_uptime(
        &self,
        identity_key: &IdentityKey,
    ) -> DomainFuture<Uptime, Self::GetRawFut> {
        self.get(&state_key::uptime_by_validator(identity_key))
    }

    // Tendermint validators are referenced to us by their Tendermint consensus key,
    // but we reference them by their Penumbra identity key.
    async fn get_validator_by_consensus_key(&self, ck: &PublicKey) -> Result<Option<Validator>> {
        if let Some(identity_key) = self
            .get(&state_key::validator_id_by_consensus_key(ck))
            .await?
        {
            self.get_validator_definition(&identity_key).await
        } else {
            return Ok(None);
        }
    }

    async fn get_validator_by_tendermint_address(
        &self,
        address: &[u8; 20],
    ) -> Result<Option<Validator>> {
        if let Some(consensus_key) = self
            .get(&state_key::consensus_key_by_tendermint_address(address))
            .await?
        {
            self.get_validator_by_consensus_key(&consensus_key).await
        } else {
            return Ok(None);
        }
    }

    /// Compute the number of epochs that will elapse before the validator is unbonded.
    // TODO(erwan): move this to the `ValidatorManager`
    async fn compute_unbonding_delay_for_validator(
        &self,
        current_epoch: Epoch,
        validator_identity: &IdentityKey,
    ) -> Result<u64> {
        let Some(val_bonding_state) = self.get_validator_bonding_state(validator_identity).await?
        else {
            anyhow::bail!(
                "validator bonding state not tracked (validator_identity={})",
                validator_identity
            )
        };

        let min_epoch_delay = self.get_stake_params().await?.unbonding_epochs;

        let epoch_delay = match val_bonding_state {
            Bonded => min_epoch_delay,
            Unbonding { unbonds_at_epoch } => unbonds_at_epoch.saturating_sub(current_epoch.index),
            Unbonded => 0u64,
        };

        // When the minimum delay parameter changes, an unbonding validator may
        // have a delay that is larger than the new minimum delay. In this case,
        // we want to use the new minimum delay.
        Ok(std::cmp::min(epoch_delay, min_epoch_delay))
    }

    /// Return the epoch index at which the validator will be unbonded.
    /// This is the minimum of the default unbonding epoch and the validator's
    /// unbonding epoch.
    async fn compute_unbonding_epoch_for_validator(&self, id: &IdentityKey) -> Result<u64> {
        let current_epoch = self.get_current_epoch().await?;
        let unbonding_delay = self
            .compute_unbonding_delay_for_validator(current_epoch, id)
            .await?;
        let unbonding_epoch = current_epoch.index.saturating_add(unbonding_delay);
        Ok(unbonding_epoch)
    }

    // TODO(erwan): we pull the entire validator definition instead of tracking
    // the consensus key separately.  If we did, not only could we save on deserialization
    // but we could also make this a clean [`DomainFuture`].
    fn fetch_validator_consensus_key(
        &self,
        identity_key: &IdentityKey,
    ) -> Pin<Box<dyn Future<Output = Result<Option<PublicKey>>> + Send + 'static>> {
        use futures::TryFutureExt;
        self.get(&state_key::validators::definitions::by_id(&identity_key))
            .map_ok(|opt: Option<Validator>| opt.map(|v: Validator| v.consensus_key))
            .boxed()
    }

    /// Returns a list of **all** known validators metadata.
    async fn validator_definitions(&self) -> Result<Vec<Validator>> {
        self.prefix(state_key::validators::definitions::prefix())
            .map_ok(|(_key, validator)| validator)
            .try_collect()
            .await
    }
}

impl<T: StateRead + ?Sized> ValidatorDataRead for T {}

#[async_trait]
pub trait ValidatorStore: StateWrite {
    fn set_validator_uptime(&mut self, identity_key: &IdentityKey, uptime: Uptime) {
        self.put(state_key::uptime_by_validator(identity_key), uptime);
    }

    fn set_validator_bonding_state(
        &mut self,
        identity_key: &IdentityKey,
        state: validator::BondingState,
    ) {
        tracing::debug!(?state, validator_identity = %identity_key, "set bonding state for validator");
        self.put(state_key::bonding_state_by_validator(identity_key), state);
    }

    #[instrument(skip(self))]
    fn set_validator_power(
        &mut self,
        identity_key: &IdentityKey,
        voting_power: Amount,
    ) -> Result<()> {
        tracing::debug!(validator_identity = ?identity_key, ?voting_power, "setting validator power");
        if voting_power.value() > MAX_VOTING_POWER {
            anyhow::bail!("voting power exceeds maximum")
        }
        self.put(state_key::power_by_validator(identity_key), voting_power);

        Ok(())
    }

    #[instrument(skip(self))]
    fn set_initial_validator_state(
        &mut self,
        id: &IdentityKey,
        initial_state: State,
    ) -> Result<()> {
        tracing::debug!(validator_identity = %id, ?initial_state, "setting initial validator state");
        if !matches!(initial_state, State::Active | State::Defined) {
            anyhow::bail!("invalid initial validator state");
        }

        self.put(state_key::state_by_validator(id), initial_state);
        Ok(())
    }

    #[instrument(skip(self))]
    fn set_validator_rate_data(&mut self, identity_key: &IdentityKey, rate_data: RateData) {
        tracing::debug!("setting validator rates");
        self.put(
            state_key::current_rate_by_validator(identity_key),
            rate_data,
        );
    }
}

impl<T: StateWrite + ?Sized> ValidatorStore for T {}
