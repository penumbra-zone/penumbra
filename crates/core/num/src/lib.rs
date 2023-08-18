#![deny(clippy::unwrap_used)]
mod allocate;
mod amount;
pub mod fixpoint;

pub use allocate::allocate;
pub use amount::{Amount, AmountVar};
