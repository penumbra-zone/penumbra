#![deny(clippy::unwrap_used)]
mod action_handler;
mod community_pool_ext;
mod mock_client;
mod state_delta_wrapper;
mod temp_storage_ext;

pub use action_handler::ActionHandler;
pub use app::StateWriteExt;
pub use community_pool_ext::CommunityPoolStateReadExt;
pub use mock_client::MockClient;
pub use temp_storage_ext::TempStorageExt;

use once_cell::sync::Lazy;

pub static SUBSTORE_PREFIXES: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        penumbra_ibc::IBC_SUBSTORE_PREFIX.to_string(),
        penumbra_chain::COMETBFT_SUBSTORE_PREFIX.to_string(),
    ]
});

pub const APP_VERSION: u64 = 1;

pub mod app;
pub mod genesis;
pub mod params;

pub mod metrics;
pub mod rpc;
pub use self::metrics::register_metrics;

#[cfg(test)]
mod tests;

/// Temporary compat wrapper for duplicate trait impls
pub struct Compat<'a, T>(&'a T);
