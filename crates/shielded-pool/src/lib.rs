#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;

pub mod event;
pub mod state_key;

pub mod output;
pub mod spend;

pub use output::{Output, OutputPlan, OutputView};
pub use spend::{Spend, SpendPlan, SpendView};
