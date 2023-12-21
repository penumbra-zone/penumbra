//! Source code for the Penumbra node software.
#![allow(clippy::clone_on_copy)]
#![deny(clippy::unwrap_used)]
#![recursion_limit = "512"]

mod consensus;
mod ibc;
mod info;
mod mempool;
mod metrics;
mod snapshot;

pub mod auto_https;
pub mod events;
pub mod testnet;
pub mod upgrade;

pub use crate::metrics::register_metrics;
pub use consensus::Consensus;
pub use ibc::{SnapshotWrapper, StorageWrapper};
pub use info::Info;
pub use mempool::Mempool;
pub use penumbra_app::app::App;
pub use snapshot::Snapshot;
