#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
mod amount;
#[cfg(feature = "r1cs")]
pub mod fixpoint;
mod percentage;

pub use amount::Amount;
#[cfg(feature = "r1cs")]
pub use amount::AmountVar;
