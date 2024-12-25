use {
    async_trait::async_trait,
    cnidarium::TempStorage,
    penumbra_sdk_app::{app::App, genesis::AppState, SUBSTORE_PREFIXES},
    std::ops::Deref,
};

#[async_trait]
pub trait TempStorageExt: Sized {
    async fn apply_genesis(self, genesis: AppState) -> anyhow::Result<Self>;
    #[allow(dead_code)]
    async fn apply_default_genesis(self) -> anyhow::Result<Self>;
    async fn new_with_penumbra_prefixes() -> anyhow::Result<TempStorage>;
}

#[async_trait]
impl TempStorageExt for TempStorage {
    async fn new_with_penumbra_prefixes() -> anyhow::Result<TempStorage> {
        TempStorage::new_with_prefixes(SUBSTORE_PREFIXES.to_vec()).await
    }

    async fn apply_genesis(self, genesis: AppState) -> anyhow::Result<Self> {
        // Check that we haven't already applied a genesis state:
        if self.latest_version() != u64::MAX {
            anyhow::bail!("database already initialized");
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
