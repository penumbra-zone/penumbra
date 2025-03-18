#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod app;
pub mod event;
pub mod genesis;
pub mod metrics;
pub mod params;
pub mod rpc;
pub mod server;

mod action_handler;
mod community_pool_ext;
mod penumbra_host_chain;

pub use crate::{
    action_handler::AppActionHandler, app::StateWriteExt,
    community_pool_ext::CommunityPoolStateReadExt, metrics::register_metrics,
    penumbra_host_chain::PenumbraHost,
};

use once_cell::sync::Lazy;

/// Representation of the Penumbra application version. Notably, this is distinct
/// from the crate version(s). This number should only ever be incremented.
pub const APP_VERSION: u64 = 7;

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
