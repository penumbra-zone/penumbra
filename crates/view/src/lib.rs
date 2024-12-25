//! The view RPC library for the Penumbra Zone.
//!
//! This crate provides a [`ViewClient`] and a [`ViewServer`]. These form a client-server pair to
//! synchronize and interact with public chain state using one or more full viewing keys. See the
//! documentation of [`ViewClient`] and a [`ViewServer`] for more information.
//!
//! This crate also provides a [`Planner`]. This is a planner for
//! [`TransactionPlan`][penumbra_sdk_transaction::TransactionPlan].
//!
//! Finally, this crate provides a [`Storage`] type for managing persistent sqlite storage.

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

pub use crate::client::ViewClient;
pub use crate::metrics::register_metrics;
pub use crate::note_record::SpendableNoteRecord;
pub use crate::planner::Planner;
pub use crate::service::ViewServer;
pub use crate::status::StatusStreamResponse;
pub use crate::storage::Storage;
pub use crate::swap_record::SwapRecord;
pub use crate::transaction_info::TransactionInfo;
