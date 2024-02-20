//! Shared integration testing facilities.

// NB: Allow dead code, these are in fact shared by files in `tests/`.
#![allow(dead_code)]

use async_trait::async_trait;
use cnidarium::TempStorage;
use penumbra_app::{
    app::App,
    server::consensus::{Consensus, ConsensusService},
};
use penumbra_genesis::AppState;
use penumbra_mock_consensus::TestNode;
use std::ops::Deref;

// Installs a tracing subscriber to log events until the returned guard is dropped.
pub fn set_tracing_subscriber() -> tracing::subscriber::DefaultGuard {
    use tracing_subscriber::filter::EnvFilter;

    let filter = "debug,penumbra_app=trace,penumbra_mock_consensus=trace";
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(filter))
        .expect("should have a valid filter directive")
        // Without explicitly disabling the `r1cs` target, the ZK proof implementations
        // will spend an enormous amount of CPU and memory building useless tracing output.
        .add_directive(
            "r1cs=off"
                .parse()
                .expect("rics=off is a valid filter directive"),
        );

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .pretty()
        .with_test_writer()
        .finish();

    tracing::subscriber::set_default(subscriber)
}

/// A [`TestNode`] coupled with Penumbra's [`Consensus`] service.
pub type PenumbraTestNode = TestNode<ConsensusService>;

/// Returns a new [`PenumbraTestNode`] backed by the given temporary storage.
pub async fn start_test_node(storage: &TempStorage) -> anyhow::Result<PenumbraTestNode> {
    use tap::TapFallible;
    let app_state = AppState::default();
    let consensus = Consensus::new(storage.as_ref().clone());
    TestNode::builder()
        .single_validator()
        .with_penumbra_auto_app_state(app_state)?
        .init_chain(consensus)
        .await
        .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))
}

#[async_trait]
pub trait TempStorageExt: Sized {
    async fn apply_genesis(self, genesis: AppState) -> anyhow::Result<Self>;
    async fn apply_default_genesis(self) -> anyhow::Result<Self>;
}

#[async_trait]
impl TempStorageExt for TempStorage {
    async fn apply_genesis(self, genesis: AppState) -> anyhow::Result<Self> {
        // Check that we haven't already applied a genesis state:
        if self.latest_version() != u64::MAX {
            anyhow::bail!("database already initialized");
        }

        // Apply the genesis state to the storage
        let mut app = App::new(self.latest_snapshot()).await?;
        app.init_chain(&genesis).await;
        app.commit(self.deref().clone()).await;

        Ok(self)
    }

    async fn apply_default_genesis(self) -> anyhow::Result<Self> {
        self.apply_genesis(Default::default()).await
    }
}

/// Penumbra-specific extensions to the mock consensus builder.
pub trait BuilderExt: Sized {
    type Error;
    fn with_penumbra_auto_app_state(self, app_state: AppState) -> Result<Self, Self::Error>;
}

impl BuilderExt for penumbra_mock_consensus::builder::Builder {
    type Error = anyhow::Error;
    fn with_penumbra_auto_app_state(self, app_state: AppState) -> Result<Self, Self::Error> {
        // what to do here?
        // - read out list of abci/comet validators from the builder,
        // - define a penumbra validator for each one
        // - inject that into the penumbra app state
        // - serialize to json and then call `with_app_state_bytes`
        let app_state = serde_json::to_vec(&app_state)?;
        Ok(self.app_state(app_state))
    }
}
