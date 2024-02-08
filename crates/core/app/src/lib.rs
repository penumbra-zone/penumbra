#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod app;
pub mod genesis;
pub mod metrics;
pub mod params;
pub mod rpc;

mod action_handler;
mod community_pool_ext;
mod mock_client;
mod penumbra_host_chain;
mod temp_storage_ext;
mod mock_comet_ext;

#[cfg(test)]
mod tests;

pub use crate::{
    action_handler::ActionHandler, app::StateWriteExt,
    community_pool_ext::CommunityPoolStateReadExt, metrics::register_metrics,
    mock_client::MockClient, penumbra_host_chain::PenumbraHost, temp_storage_ext::TempStorageExt,
};

use once_cell::sync::Lazy;

pub const APP_VERSION: u64 = 1;

pub static SUBSTORE_PREFIXES: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        penumbra_ibc::IBC_SUBSTORE_PREFIX.to_string(),
        COMETBFT_SUBSTORE_PREFIX.to_string(),
    ]
});

/// The substore prefix used for storing historical CometBFT block data.
pub static COMETBFT_SUBSTORE_PREFIX: &'static str = "cometbft-data";

/// Temporary compat wrapper for duplicate trait impls
pub struct Compat<'a, T>(&'a T);
