//! Source code for the Penumbra node software.

// This is for the async_stream macros
#![recursion_limit = "1024"]
#![allow(clippy::clone_on_copy)]

mod consensus;
mod db;
mod info;
mod mempool;
mod pd_metrics;
mod pending_block;
mod request_ext;
mod snapshot;
mod validator_set;
mod verify;
mod wallet;

pub mod genesis;
pub mod state;
pub mod testnet;

pub use consensus::Consensus;
pub use info::Info;
pub use mempool::Mempool;
pub use pd_metrics::register_all_metrics;
use pending_block::PendingBlock;
use request_ext::RequestExt;
pub use snapshot::Snapshot;

/// The age limit, in blocks, on anchors accepted in transaction verification.
pub const NUM_RECENT_ANCHORS: usize = 256;
