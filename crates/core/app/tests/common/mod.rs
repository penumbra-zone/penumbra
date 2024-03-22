//! Shared integration testing facilities.

// NB: Allow dead code, and unused imports. these are shared and consumed by files in `tests/`.
#![allow(dead_code, unused_imports)]

pub use self::test_node_builder_ext::BuilderExt;

use {
    async_trait::async_trait,
    cnidarium::TempStorage,
    penumbra_app::{
        app::App,
        server::consensus::{Consensus, ConsensusService},
    },
    penumbra_genesis::AppState,
    penumbra_mock_consensus::TestNode,
    std::ops::Deref,
};

/// Penumbra-specific extensions to the mock consensus builder.
///
/// See [`BuilderExt`].
mod test_node_builder_ext;

// Installs a tracing subscriber to log events until the returned guard is dropped.
pub fn set_tracing_subscriber() -> tracing::subscriber::DefaultGuard {
    use tracing_subscriber::filter::EnvFilter;

    let filter = "info,penumbra_app=trace,penumbra_mock_consensus=trace";
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
