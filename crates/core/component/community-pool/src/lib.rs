#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;

pub mod event;

mod action;
pub use action::{CommunityPoolDeposit, CommunityPoolOutput, CommunityPoolSpend};

pub mod genesis;
pub mod params;

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub use component::{StateReadExt, StateWriteExt};
