#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;

pub mod event;
pub mod state_key;

mod fee;
mod gas;
pub mod genesis;
pub mod params;

pub use fee::{Fee, FeeTier};
pub use gas::{Gas, GasPrices};
pub use params::FeeParameters;
