//! Source code for the Penumbra node software.
#![allow(clippy::clone_on_copy)]
#![recursion_limit = "512"]

mod consensus;
mod info;
mod mempool;
mod metrics;
mod snapshot;

pub mod auto_https;
pub mod testnet;

pub use crate::metrics::register_metrics;
pub use consensus::Consensus;
pub use info::Info;
pub use mempool::Mempool;
pub use penumbra_app::app::App;
pub use snapshot::Snapshot;
