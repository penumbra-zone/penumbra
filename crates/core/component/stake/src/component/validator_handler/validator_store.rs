use crate::{
    component::{StateReadExt as _, MAX_VOTING_POWER},
    rate::RateData,
    state_key,
    validator::{self, BondingState::*, State, Validator},
    IdentityKey, Uptime,
};
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use futures::{Future, FutureExt, TryStreamExt};
use penumbra_num::Amount;
use penumbra_proto::{state::future::DomainFuture, DomainType, StateReadProto, StateWriteProto};
use std::pin::Pin;
use tendermint::PublicKey;
use tracing::instrument;

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
        self.get(&state_key::validators::state::by_id(identity_key))
    }

    async fn get_validator_bonding_state(
        &self,
        identity_key: &IdentityKey,
    ) -> Option<validator::BondingState> {
        self.get(&state_key::validators::pool::bonding_state::by_id(
            identity_key,
        ))
        .await
        .expect("no deserialization error expected")
    }

    /// Returns the amount of delegation tokens in the specified validator's pool.
    async fn get_validator_pool_size(&self, identity_key: &IdentityKey) -> Option<Amount> {
        self.get(&state_key::validators::pool::balance::by_id(identity_key))
            .await
            .expect("no deserialization error expected")
    }

    /// Convenience method to assemble a [`ValidatorStatus`](crate::validator::Status).
    async fn get_validator_status(
        &self,
        identity_key: &IdentityKey,
    ) -> Result<Option<validator::Status>> {
        let bonding_state = self.get_validator_bonding_state(identity_key).await;
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
        self.get(&state_key::validators::rate::current_by_id(identity_key))
            .boxed()
    }

    async fn get_prev_validator_rate(&self, identity_key: &IdentityKey) -> Option<RateData> {
        self.get(&state_key::validators::rate::previous_by_id(identity_key))
            .await
            .expect("no deserialization error expected")
    }

    fn get_validator_power(
        &self,
        validator: &IdentityKey,
    ) -> DomainFuture<Amount, Self::GetRawFut> {
        self.get(&state_key::validators::power::by_id(validator))
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
        let key = state_key::validators::uptime::by_id(identity_key);
        self.nonverifiable_get(key.as_bytes())
    }

    // Tendermint validators are referenced to us by their Tendermint consensus key,
    // but we reference them by their Penumbra identity key.
    async fn get_validator_by_consensus_key(&self, ck: &PublicKey) -> Result<Option<Validator>> {
        if let Some(identity_key) = self
            .get(&state_key::validators::lookup_by::consensus_key(ck))
            .await?
        {
            self.get_validator_definition(&identity_key).await
        } else {
            return Ok(None);
        }
    }

    async fn get_validator_by_cometbft_address(
        &self,
        address: &[u8; 20],
    ) -> Result<Option<Validator>> {
        if let Some(consensus_key) = self
            .get(&state_key::validators::lookup_by::cometbft_address(address))
            .await?
        {
            self.get_validator_by_consensus_key(&consensus_key).await
        } else {
            return Ok(None);
        }
    }

    /// Compute the unbonding height for an undelegation initiated at `start_height`,
    /// relative to the **current** state of the validator pool.
    ///
    /// Returns `None` if the pool is [`Unbonded`](crate::validator::State).
    ///
    /// This can be used to check if the undelegation is allowed, or compute a penalty range,
    /// or to compute the epoch at which a delegation pool will be unbonded.
    async fn compute_unbonding_height(
        &self,
        id: &IdentityKey,
        start_height: u64,
    ) -> Result<Option<u64>> {
        let Some(val_bonding_state) = self.get_validator_bonding_state(id).await else {
            anyhow::bail!(
                "validator bonding state not tracked (validator_identity={})",
                id
            )
        };

        let min_block_delay = self.get_stake_params().await?.unbonding_delay;
        let upper_bound_height = start_height.saturating_add(min_block_delay);

        let unbonding_height = match val_bonding_state {
            // The pool is bonded, so the unbonding height is the start height plus the delay.
            Bonded => Some(upper_bound_height),
            // The pool is unbonding at a specific height, so we can use that.
            Unbonding { unbonds_at_height } => {
                if unbonds_at_height > start_height {
                    Some(unbonds_at_height)
                } else {
                    // In some cases, the allowed unbonding height can be smaller than
                    // undelgation start height, for example if the unbonding delay has
                    // changed in a parameter update, or if the unbonding has finished
                    // and the validator is not indexed by the staking module anymore.
                    // This is functionally equivalent to dealing with an `Unbonded` pool.
                    None
                }
            }
            // The pool is unbonded, so the unbonding height can be decided by the caller.
            Unbonded => None,
        };

        Ok(unbonding_height)
    }

    // TODO(erwan): we pull the entire validator definition instead of tracking
    // the consensus key separately.  If we did, not only could we save on deserialization
    // but we could also make this a clean [`DomainFuture`].
    fn fetch_validator_consensus_key(
        &self,
        identity_key: &IdentityKey,
    ) -> Pin<Box<dyn Future<Output = Result<Option<PublicKey>>> + Send + 'static>> {
        use futures::TryFutureExt;
        self.get(&state_key::validators::definitions::by_id(identity_key))
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
pub(crate) trait ValidatorDataWrite: StateWrite {
    fn set_validator_uptime(&mut self, identity_key: &IdentityKey, uptime: Uptime) {
        self.nonverifiable_put_raw(
            state_key::validators::uptime::by_id(identity_key)
                .as_bytes()
                .to_vec(),
            uptime.encode_to_vec(),
        );
    }

    fn set_validator_bonding_state(
        &mut self,
        identity_key: &IdentityKey,
        state: validator::BondingState,
    ) {
        tracing::debug!(?state, validator_identity = %identity_key, "set bonding state for validator");
        self.put(
            state_key::validators::pool::bonding_state::by_id(identity_key),
            state,
        );
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
        self.put(
            state_key::validators::power::by_id(identity_key),
            voting_power,
        );

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

        self.put(state_key::validators::state::by_id(id), initial_state);
        Ok(())
    }

    #[instrument(skip(self))]
    fn set_validator_rate_data(&mut self, identity_key: &IdentityKey, rate_data: RateData) {
        tracing::debug!("setting validator rate data");
        self.put(
            state_key::validators::rate::current_by_id(identity_key),
            rate_data,
        );
    }

    #[instrument(skip(self))]
    /// Persist the previous validator rate data, inclusive of accumulated penalties.
    fn set_prev_validator_rate(&mut self, identity_key: &IdentityKey, rate_data: RateData) {
        let path = state_key::validators::rate::previous_by_id(identity_key);
        self.put(path, rate_data)
    }
}

impl<T: StateWrite + ?Sized> ValidatorDataWrite for T {}

#[async_trait]
pub(crate) trait ValidatorPoolTracker: StateWrite {
    /// Set the validator pool size, overwriting any existing value.
    fn set_validator_pool_size(&mut self, identity_key: &IdentityKey, amount: Amount) {
        self.put(
            state_key::validators::pool::balance::by_id(identity_key),
            amount,
        );
    }

    /// Checked increase of the validator pool size by the given amount.
    /// Returns the new pool size, or `None` if the update failed.
    async fn increase_validator_pool_size(
        &mut self,
        identity_key: &IdentityKey,
        add: Amount,
    ) -> Option<Amount> {
        let state_path = state_key::validators::pool::balance::by_id(identity_key);
        let old_supply = self
            .get(&state_path)
            .await
            .expect("no deserialization error expected")
            .unwrap_or(Amount::zero());

        tracing::debug!(validator_identity = %identity_key, ?add, ?old_supply, "expanding validator pool size");

        if let Some(new_supply) = old_supply.checked_add(&add) {
            self.put(state_path, new_supply);
            Some(new_supply)
        } else {
            None
        }
    }

    /// Checked decrease of the validator pool size by the given amount.
    /// Returns the new pool size, or `None` if the update failed.
    async fn decrease_validator_pool_size(
        &mut self,
        identity_key: &IdentityKey,
        sub: Amount,
    ) -> Option<Amount> {
        let state_path = state_key::validators::pool::balance::by_id(identity_key);
        let old_supply = self
            .get(&state_path)
            .await
            .expect("no deserialization error expected")
            .unwrap_or(Amount::zero());

        tracing::debug!(validator_identity = %identity_key, ?sub, ?old_supply, "contracting validator pool size");

        if let Some(new_supply) = old_supply.checked_sub(&sub) {
            self.put(state_path, new_supply);
            Some(new_supply)
        } else {
            None
        }
    }
}

impl<T: StateWrite + ?Sized> ValidatorPoolTracker for T {}
