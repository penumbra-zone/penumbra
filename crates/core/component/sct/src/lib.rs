#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;

pub mod event;
mod nullification_info;
mod nullifier;
mod source;
pub mod state_key;

pub use nullification_info::NullificationInfo;
pub use nullifier::{Nullifier, NullifierVar};
pub use source::CommitmentSource;
