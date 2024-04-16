//! The dex component contains implementations of the Penumbra dex with token
//! supplies based on liquidity provider interactions.

pub mod metrics;
pub mod rpc;

pub mod router;

mod action_handler;
mod arb;
pub(crate) mod circuit_breaker;
mod dex;
mod flow;
mod position_manager;
mod swap_manager;

pub use self::metrics::register_metrics;
pub(crate) use arb::Arbitrage;
pub use circuit_breaker::ExecutionCircuitBreaker;
pub(crate) use circuit_breaker::ValueCircuitBreaker;
pub use dex::{Dex, StateReadExt, StateWriteExt};
pub use position_manager::{PositionManager, PositionRead};
pub use swap_manager::SwapManager;
#[cfg(test)]
pub(crate) mod tests;
