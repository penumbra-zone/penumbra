use std::{collections::BTreeMap, pin::Pin};

use crate::{
    component::metrics,
    rate::{BaseRateData, RateData},
    validator::{State, Validator},
    DelegationToken,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::{Future, FutureExt, StreamExt as _};
use penumbra_num::Amount;
use penumbra_sct::component::clock::{EpochManager, EpochRead};
use penumbra_shielded_pool::component::{SupplyRead as _, SupplyWrite};
use sha2::{Digest as _, Sha256};
use tendermint::abci::types::{CommitInfo, Misbehavior};
use tokio::task::JoinSet;
use validator::BondingState::*;

use cnidarium::{StateRead, StateWrite};
use penumbra_proto::{state::future::DomainFuture, StateReadProto, StateWriteProto};
use tracing::instrument;

use crate::{
    component::StakingDataRead,
    component::StateWriteExt as _,
    state_key,
    validator::{self},
    IdentityKey, Penalty, StateReadExt as _, Uptime,
};
use penumbra_asset::asset;
use crate::component::MAX_VOTING_POWER;
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
