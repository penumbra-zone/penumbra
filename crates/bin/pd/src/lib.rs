//! Source code for the Penumbra node software.
#![allow(clippy::clone_on_copy)]
#![deny(clippy::unwrap_used)]
#![recursion_limit = "512"]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod metrics;

pub mod cli;
pub mod migrate;
pub mod network;
pub mod zipserve;

pub use crate::metrics::register_metrics;
pub use penumbra_sdk_app::app::App;

pub const MINIFRONT_ARCHIVE_BYTES: &[u8] = include_bytes!("../../../../assets/minifront.zip");

pub const NODE_STATUS_ARCHIVE_BYTES: &[u8] = include_bytes!("../../../../assets/node-status.zip");
