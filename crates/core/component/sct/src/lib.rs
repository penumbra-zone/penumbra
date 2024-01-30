#![deny(clippy::unwrap_used)]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#[cfg(feature = "component")]
pub mod component;

pub mod event;
mod nullification_info;
mod nullifier;
mod source;
pub mod state_key;
pub mod params

pub use nullification_info::NullificationInfo;
pub use nullifier::{Nullifier, NullifierVar};
pub use source::CommitmentSource;
