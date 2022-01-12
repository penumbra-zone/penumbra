//! Source code for the Penumbra node software.

// This is for the async_stream macros
#![recursion_limit = "512"]

mod app;
mod db;
mod pd_metrics;
mod pending_block;
mod request_ext;
mod sequential;
mod state;
mod verify;
mod wallet;

use sequential::Sequencer;

pub mod genesis;
pub mod testnet;

pub use app::App;
pub use pd_metrics::register_all_metrics;
pub use pending_block::PendingBlock;
pub use request_ext::RequestExt;
pub use state::State;
