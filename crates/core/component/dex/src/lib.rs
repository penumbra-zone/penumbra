#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;
pub mod event;
pub mod state_key;

mod batch_swap_output_data;
mod swap_execution;
mod trading_pair;

pub use batch_swap_output_data::BatchSwapOutputData;
pub use swap_execution::SwapExecution;
pub use trading_pair::{DirectedTradingPair, DirectedUnitPair, TradingPair, TradingPairVar};

pub mod lp;
pub mod swap;
pub mod swap_claim;
