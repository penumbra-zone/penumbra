#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
mod amount;
pub mod fixpoint;
mod percentage;

pub use amount::{Amount, AmountVar};
pub use percentage::Percentage;
