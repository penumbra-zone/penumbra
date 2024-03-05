#![deny(clippy::unwrap_used)]
#![recursion_limit = "512"]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
mod client;
mod metrics;
mod note_record;
mod planner;
mod service;
mod status;
mod storage;
mod swap_record;
mod sync;
mod transaction_info;
mod worker;

use worker::Worker;

pub use crate::metrics::register_metrics;
pub use client::{BroadcastStatusStream, ViewClient};
pub use note_record::SpendableNoteRecord;
pub use planner::Planner;
pub use service::ViewServer;
pub use status::StatusStreamResponse;
pub use storage::Storage;
pub use swap_record::SwapRecord;
pub use transaction_info::TransactionInfo;
