#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#[cfg(feature = "component")]
pub mod component;

pub mod event;

mod action;
pub use action::{CommunityPoolDeposit, CommunityPoolOutput, CommunityPoolSpend};

pub mod genesis;
pub mod params;

#[cfg(feature = "component")]
pub use component::{StateReadExt, StateWriteExt};
