//! Shared integration testing facilities.

// NB: Allow dead code, and unused imports. these are shared and consumed by files in `tests/`.
#![allow(dead_code, unused_imports)]

pub use self::{temp_storage_ext::TempStorageExt, test_node_builder_ext::BuilderExt};

use {
    async_trait::async_trait,
    cnidarium::TempStorage,
    penumbra_app::{
        app::App,
        genesis::AppState,
        server::consensus::{Consensus, ConsensusService},
    },
    penumbra_mock_consensus::TestNode,
    std::ops::Deref,
};

/// Penumbra-specific extensions to the mock consensus builder.
///
/// See [`BuilderExt`].
mod test_node_builder_ext;

/// Extensions to [`TempStorage`][cnidarium::TempStorage].
mod temp_storage_ext;

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

#[async_trait]
pub trait TestNodeExt: Sized {
    async fn fast_forward_to_next_epoch(
        &mut self,
        storage: &TempStorage,
    ) -> anyhow::Result<penumbra_sct::epoch::Epoch>;
}

#[async_trait]
impl<C> TestNodeExt for TestNode<C>
where
    C: tower::Service<
            tendermint::v0_37::abci::ConsensusRequest,
            Response = tendermint::v0_37::abci::ConsensusResponse,
            Error = tower::BoxError,
        > + Send
        + Clone
        + 'static,
    C::Future: Send + 'static,
    C::Error: Sized,
{
    async fn fast_forward_to_next_epoch(
        &mut self,
        storage: &TempStorage,
    ) -> Result<penumbra_sct::epoch::Epoch, anyhow::Error> {
        use {penumbra_sct::component::clock::EpochRead, tap::Tap};

        let get_epoch = || async { storage.latest_snapshot().get_current_epoch().await };
        let start = get_epoch()
            .await?
            .tap(|start| tracing::info!(?start, "fast forwarding to next epoch"));

        loop {
            self.block().execute().await?;
            let current = get_epoch().await?;
            if current != start {
                tracing::debug!(end = ?current, ?start, "reached next epoch");
                return Ok(current);
            }
        }
    }
}
