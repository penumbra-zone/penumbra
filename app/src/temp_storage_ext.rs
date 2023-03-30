use std::ops::Deref;

use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_storage::TempStorage;

use crate::app::App;

#[async_trait]
pub trait TempStorageExt: Sized {
    async fn apply_genesis(self, genesis: genesis::AppState) -> anyhow::Result<Self>;
    async fn apply_default_genesis(self) -> anyhow::Result<Self>;
}

#[async_trait]
impl TempStorageExt for TempStorage {
    async fn apply_genesis(self, genesis: genesis::AppState) -> anyhow::Result<Self> {
        // Check that we haven't already applied a genesis state:
        if self.latest_version() != u64::MAX {
            return Err(anyhow::anyhow!("database already initialized"));
        }

        // Apply the genesis state to the storage
        let mut app = App::new(self.latest_snapshot());
        app.init_chain(&genesis).await;
        app.commit(self.deref().clone()).await;

        Ok(self)
    }

    async fn apply_default_genesis(self) -> anyhow::Result<Self> {
        self.apply_genesis(Default::default()).await
    }
}
