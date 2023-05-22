//! The dex component contains implementations of the Penumbra dex with token supplies based on liquidity provider interactions.
pub mod metrics;

pub mod router;

mod action_handler;
mod arb;
mod dex;
mod position_manager;
mod swap_manager;

pub use self::metrics::register_metrics;
pub use arb::Arbitrage;
pub use dex::{Dex, StateReadExt, StateWriteExt};
pub use position_manager::{PositionManager, PositionRead};
pub use swap_manager::SwapManager;

#[cfg(test)]
mod tests;
