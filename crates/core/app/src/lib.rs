#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod action_handler;
mod community_pool_ext;
mod mock_client;
mod penumbra_host_chain;
mod temp_storage_ext;

pub use action_handler::ActionHandler;
pub use app::StateWriteExt;
pub use community_pool_ext::CommunityPoolStateReadExt;
pub use mock_client::MockClient;
pub use penumbra_host_chain::PenumbraHost;
pub use temp_storage_ext::TempStorageExt;

use once_cell::sync::Lazy;

pub static SUBSTORE_PREFIXES: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        penumbra_ibc::IBC_SUBSTORE_PREFIX.to_string(),
       COMETBFT_SUBSTORE_PREFIX.to_string(),
    ]
});

pub mod app;
pub mod genesis;
pub mod params;

pub mod metrics;
pub mod rpc;
pub use self::metrics::register_metrics;

pub const APP_VERSION: u64 = 1;
/// The substore prefix used for storing histori CometBFT block data.
pub static COMETBFT_SUBSTORE_PREFIX: &'static str = "cometbft-data";

#[cfg(test)]
mod tests;

/// Temporary compat wrapper for duplicate trait impls
pub struct Compat<'a, T>(&'a T);
