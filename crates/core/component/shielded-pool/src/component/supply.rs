use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_asset::asset::{self, Metadata};
use penumbra_proto::{StateReadProto, StateWriteProto};

use tracing::instrument;

use crate::state_key;

#[async_trait]
pub trait SupplyRead: StateRead {
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
}

impl<T: StateWrite + ?Sized> SupplyWrite for T {}
