//! Shared integration testing facilities.

// NB: these reëxports are shared and consumed by files in `tests/`.
#[allow(unused_imports)]
pub use {
    self::{
        temp_storage_ext::TempStorageExt, test_node_builder_ext::BuilderExt,
        test_node_ext::TestNodeExt, validator_read_ext::ValidatorDataReadExt,
    },
    penumbra_sdk_test_subscriber::{
        set_tracing_subscriber, set_tracing_subscriber_with_env_filter,
    },
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

/// Helpful additions for reading validator information.
///
/// See [`ValidatorDataRead`][penumbra_sdk_stake::component::validator_handler::ValidatorDataRead],
/// and [`ValidatorDataReadExt`].
mod validator_read_ext;

/// Methods for testing IBC functionality.
#[allow(unused)]
pub mod ibc_tests;
