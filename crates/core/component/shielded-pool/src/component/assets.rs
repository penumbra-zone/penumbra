use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_asset::asset::{self, Metadata};
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};

use tracing::instrument;

use crate::state_key;

#[async_trait]
pub trait AssetRegistryRead: StateRead {
    async fn denom_metadata_by_asset(&self, asset_id: &asset::Id) -> Option<Metadata> {
        self.get(&state_key::denom_metadata_by_asset::by_asset_id(asset_id))
            .await
            .expect("no deserialization error")
    }
}

impl<T: StateRead + ?Sized> AssetRegistryRead for T {}

#[async_trait]
pub trait AssetRegistry: StateWrite {
    /// Register a new asset present in the shielded pool.
    /// If the asset is already registered, this is a no-op.
    #[instrument(skip(self))]
    async fn register_denom(&mut self, denom: &Metadata) {
        let asset_id = denom.id();
        tracing::debug!(?asset_id, "registering asset metadata in shielded pool");

        if self.denom_metadata_by_asset(&asset_id).await.is_none() {
            self.put(
                state_key::denom_metadata_by_asset::by_asset_id(&asset_id),
                denom.clone(),
            );
        }
    }
}

impl<T: StateWrite + ?Sized> AssetRegistry for T {}
