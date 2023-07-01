use anyhow::Result;
use async_trait::async_trait;
use penumbra_asset::asset::{self, DenomMetadata};
use penumbra_chain::KnownAssets;
use penumbra_num::Amount;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

use tracing::instrument;

use crate::state_key;

#[async_trait]
pub trait SupplyRead: StateRead {
    async fn token_supply(&self, asset_id: &asset::Id) -> Result<Option<u64>> {
        self.get_proto(&state_key::token_supply(asset_id)).await
    }

    // TODO: refactor for new state model -- no more list of known asset IDs with fixed key
    async fn known_assets(&self) -> Result<KnownAssets> {
        Ok(self
            .get(state_key::known_assets())
            .await?
            .unwrap_or_default())
    }

    async fn denom_by_asset(&self, asset_id: &asset::Id) -> Result<Option<DenomMetadata>> {
        self.get(&state_key::denom_by_asset(asset_id)).await
    }
}

impl<T: StateRead + ?Sized> SupplyRead for T {}

#[async_trait]
pub trait SupplyWrite: StateWrite {
    // TODO: refactor for new state model -- no more list of known asset IDs with fixed key
    #[instrument(skip(self))]
    async fn register_denom(&mut self, denom: &DenomMetadata) -> Result<()> {
        let id = denom.id();
        if self.denom_by_asset(&id).await?.is_some() {
            tracing::debug!(?denom, ?id, "skipping existing denom");
            Ok(())
        } else {
            tracing::debug!(?denom, ?id, "registering new denom");
            // We want to be able to query for the denom by asset ID...
            self.put(state_key::denom_by_asset(&id), denom.clone());
            // ... and we want to record it in the list of known asset IDs
            // (this requires reading the whole list, which is sad, but hopefully
            // we don't do this often).
            // TODO: fix with new state model
            let mut known_assets = self.known_assets().await?;

            known_assets.0.push(denom.to_owned());
            self.put(state_key::known_assets().to_owned(), known_assets);
            Ok(())
        }
    }

    // TODO: should this really be separate from note management?
    // #[instrument(skip(self, change))]
    async fn update_token_supply(&mut self, asset_id: &asset::Id, change: i128) -> Result<()> {
        let key = state_key::token_supply(asset_id);
        let current_supply: Amount = self.get_proto(&key).await?.unwrap_or(0u64).into();

        // TODO: replace with a single checked_add_signed call when mixed_integer_ops lands in stable (1.66)
        let new_supply: Amount = if change < 0 {
            current_supply
                .value()
                .checked_sub(change.unsigned_abs())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "underflow updating token {} supply {} with delta {}",
                        asset_id,
                        current_supply,
                        change
                    )
                })?
                .into()
        } else {
            current_supply
                .value()
                .checked_add(change as u128)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "overflow updating token {} supply {} with delta {}",
                        asset_id,
                        current_supply,
                        change
                    )
                })?
                .into()
        };
        tracing::debug!(?current_supply, ?new_supply, ?change);

        self.put(key, new_supply);
        Ok(())
    }
}

impl<T: StateWrite + ?Sized> SupplyWrite for T {}
