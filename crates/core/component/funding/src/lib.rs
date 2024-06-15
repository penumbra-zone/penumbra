#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#[cfg(feature = "component")]
pub mod component;
pub mod event;

pub mod genesis;
pub mod params;
pub use params::FundingParameters;
