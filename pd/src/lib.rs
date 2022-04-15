//! Source code for the Penumbra node software.

// This is for the async_stream macros
#![recursion_limit = "512"]
#![allow(clippy::clone_on_copy)]

mod components;
mod consensus;
mod info;
mod mempool;
mod pd_metrics;
mod pending_block;
mod request_ext;
mod snapshot;
mod storage;
mod verify;
mod wallet;

pub mod genesis;
pub mod testnet;

pub use components::{App, Component};
pub use consensus::Consensus;
pub use info::Info;
pub use mempool::{Mempool, OldMempool};
pub use pd_metrics::register_all_metrics;

use request_ext::RequestExt;
pub use snapshot::Snapshot;
pub use storage::{Overlay, OverlayExt, Storage};

/// The age limit, in blocks, on anchors accepted in transaction verification.
pub const NUM_RECENT_ANCHORS: usize = 256;
