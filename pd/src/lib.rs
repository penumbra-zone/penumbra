//! Source code for the Penumbra node software.
#![allow(clippy::clone_on_copy)]
#![recursion_limit = "256"]
mod consensus;
mod info;
mod mempool;
mod metrics;
mod request_ext;
mod snapshot;
mod tendermint_proxy;

pub mod testnet;
/// A vendored copy of the unpublished `tracing-tower` crate.
pub mod trace;

pub use request_ext::RequestExt;

pub use crate::metrics::register_metrics;
pub use consensus::Consensus;
pub use info::Info;
pub use mempool::Mempool;
pub use penumbra_component::app::App;
pub use snapshot::Snapshot;
pub use tendermint_proxy::TendermintProxy;
