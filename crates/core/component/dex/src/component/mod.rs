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
mod eviction_manager;
mod flow;
mod lqt;
mod position_manager;
mod swap_manager;

pub use dex::{Dex, StateReadExt, StateWriteExt};
pub use position_manager::PositionManager;

// Read data from the Dex component;
pub use lqt::LqtRead;
pub use position_manager::PositionRead;
pub use swap_manager::SwapDataRead;

pub(crate) use arb::Arbitrage;
pub(crate) use circuit_breaker::ExecutionCircuitBreaker;
pub(crate) use circuit_breaker::ValueCircuitBreaker;
pub use circuit_breaker::ValueCircuitBreakerRead;
pub(crate) use dex::InternalDexWrite;
pub(crate) use swap_manager::SwapDataWrite;
pub(crate) use swap_manager::SwapManager;

#[cfg(test)]
pub(crate) mod tests;

pub use self::metrics::register_metrics;
