//! Source code for the Penumbra node software.

// This is for the async_stream macros
#![recursion_limit = "512"]
#![allow(clippy::clone_on_copy)]

mod consensus;
mod info;
mod mempool;
mod pd_metrics;
mod request_ext;
mod snapshot;
mod storage;

pub mod components;
pub mod genesis;
pub mod testnet;

use request_ext::RequestExt;

pub use components::{App, Component};
pub use consensus::Consensus;
pub use info::Info;
pub use mempool::Mempool;
pub use pd_metrics::register_all_metrics;
pub use snapshot::Snapshot;
pub use storage::{Overlay, OverlayExt, Storage};
