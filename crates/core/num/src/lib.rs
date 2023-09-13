#![deny(clippy::unwrap_used)]
mod amount;
pub mod fixpoint;

pub use amount::{Amount, AmountVar};
