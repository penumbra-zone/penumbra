//! Shared integration testing facilities.

// NB: Allow dead code, and unused imports. these are shared and consumed by files in `tests/`.
#![allow(dead_code, unused_imports)]

pub use self::{
    temp_storage_ext::TempStorageExt, test_node_builder_ext::BuilderExt,
    test_node_ext::TestNodeExt, tracing_subscriber::set_tracing_subscriber,
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

/// A pretty [`tracing`] subscriber for use in test cases.
///
/// NB: this subscriber makes use of a test writer, that is compatible with `cargo test`'s output
/// capturing.
mod tracing_subscriber;

/// A [`TestNode`] coupled with Penumbra's [`Consensus`] service.
pub type PenumbraTestNode =
    penumbra_mock_consensus::TestNode<penumbra_app::server::consensus::ConsensusService>;
