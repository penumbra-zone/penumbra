#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
mod amount;
pub mod fixpoint;

pub use amount::{Amount, AmountVar};
