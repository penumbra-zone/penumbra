#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#[cfg(feature = "component")]
pub mod component;

pub mod genesis;
pub mod params;
pub use params::DistributionsParameters;
pub mod event;
