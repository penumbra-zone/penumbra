//! The dex component contains implementations of the Penumbra dex with token
//! supplies based on liquidity provider interactions.

pub mod metrics;
pub mod rpc;

pub mod router;

mod action_handler;
mod arb;
mod chandelier;
pub(crate) mod circuit_breaker;
mod dex;
mod flow;
mod position_manager;
mod swap_manager;

pub use self::metrics::register_metrics;
pub(crate) use arb::Arbitrage;
pub(crate) use circuit_breaker::ExecutionCircuitBreaker;
pub(crate) use circuit_breaker::ValueCircuitBreaker;
pub(crate) use dex::InternalDexWrite;
pub use dex::{Dex, StateReadExt, StateWriteExt};
pub use position_manager::PositionManager;
pub(crate) use swap_manager::SwapDataWrite;
pub(crate) use swap_manager::SwapManager;

// Read data from the Dex component;
pub use position_manager::PositionRead;
pub use swap_manager::SwapDataRead;
#[cfg(test)]
pub(crate) mod tests;
