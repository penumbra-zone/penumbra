//! Shared integration testing facilities.

// NB: Allow dead code, and unused imports. these are shared and consumed by files in `tests/`.
#![allow(dead_code, unused_imports)]

pub use self::{
    temp_storage_ext::TempStorageExt, test_node_builder_ext::BuilderExt, test_node_ext::TestNodeExt,
};

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

/// Penumbra-specific extensions to the mock consensus test node.
///
/// See [`TestNodeExt`].
mod test_node_ext;

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
