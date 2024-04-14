// Requires nightly
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(clippy::unwrap_used)]

pub mod params;
pub mod state_key;

#[cfg(feature = "component")]
pub mod component;

#[cfg(feature = "component")]
pub use component::{StateReadExt, StateWriteExt};
