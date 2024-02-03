use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_asset::asset::{self, Metadata};
use penumbra_num::Amount;
use penumbra_proto::{StateReadProto, StateWriteProto};

use tracing::instrument;

use crate::state_key;

#[async_trait]
pub trait SupplyRead: StateRead {
    async fn token_supply(&self, asset_id: &asset::Id) -> Result<Option<Amount>> {
        self.get(&state_key::token_supply(asset_id)).await
    }

    async fn denom_by_asset(&self, asset_id: &asset::Id) -> Result<Option<Metadata>> {
        self.get(&state_key::denom_by_asset(asset_id)).await
    }
}

impl<T: StateRead + ?Sized> SupplyRead for T {}

#[async_trait]
pub trait SupplyWrite: StateWrite {
    // TODO: why not make this infallible and synchronous?
    #[instrument(skip(self))]
    async fn register_denom(&mut self, denom: &Metadata) -> Result<()> {
        let id = denom.id();
        if self.denom_by_asset(&id).await?.is_some() {
            tracing::debug!(?denom, ?id, "skipping existing denom");
            Ok(())
        } else {
            tracing::debug!(?denom, ?id, "registering new denom");
            // We want to be able to query for the denom by asset ID
            self.put(state_key::denom_by_asset(&id), denom.clone());
            Ok(())
        }
    }

    async fn increase_token_supply(
        &mut self,
        asset_id: &asset::Id,
        amount_to_add: Amount,
    ) -> Result<()> {
        let key = state_key::token_supply(asset_id);
        let current_supply: Amount = self.get(&key).await?.unwrap_or(0u128.into());

        tracing::debug!(
            ?current_supply,
            ?amount_to_add,
            ?asset_id,
            "increasing token supply"
        );
        let new_supply = current_supply.checked_add(&amount_to_add).ok_or_else(|| {
            anyhow::anyhow!(
                "overflow updating token {} supply {} with delta {}",
                asset_id,
                current_supply,
                amount_to_add
            )
        })?;

        self.put(key, new_supply);
        Ok(())
    }

    async fn decrease_token_supply(
        &mut self,
        asset_id: &asset::Id,
        amount_to_sub: Amount,
    ) -> Result<()> {
        let key = state_key::token_supply(asset_id);
        let current_supply: Amount = self.get(&key).await?.unwrap_or(0u128.into());

        tracing::debug!(
            ?current_supply,
            ?amount_to_sub,
            ?asset_id,
            "decreasing token supply"
        );
        let new_supply = current_supply.checked_sub(&amount_to_sub).ok_or_else(|| {
            anyhow::anyhow!(
                "overflow updating token {} supply {} with delta {}",
                asset_id,
                current_supply,
                amount_to_sub
            )
        })?;

        self.put(key, new_supply);
        Ok(())
    }
}

impl<T: StateWrite + ?Sized> SupplyWrite for T {}
