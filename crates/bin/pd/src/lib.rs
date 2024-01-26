//! Source code for the Penumbra node software.
#![allow(clippy::clone_on_copy)]
#![deny(clippy::unwrap_used)]
#![recursion_limit = "512"]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod consensus;
mod info;
mod mempool;
mod metrics;
mod snapshot;

pub mod auto_https;
pub mod cli;
pub mod events;
pub mod migrate;
pub mod testnet;

pub use crate::metrics::register_metrics;
pub use consensus::Consensus;
pub use info::Info;
pub use mempool::Mempool;
pub use penumbra_app::app::App;
pub use snapshot::Snapshot;
