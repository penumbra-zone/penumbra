//! Source code for the Penumbra node software.
#![allow(clippy::clone_on_copy)]

mod consensus;
mod info;
mod mempool;
mod metrics;
mod request_ext;
mod snapshot;

pub mod testnet;

use request_ext::RequestExt;

pub use crate::metrics::register_metrics;
pub use consensus::Consensus;
pub use info::Info;
pub use mempool::Mempool;
pub use penumbra_component::app::App;
pub use snapshot::Snapshot;
