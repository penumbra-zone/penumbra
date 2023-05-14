//! The dex component contains implementations of the Penumbra dex with token supplies based on liquidity provider interactions.
mod component;
pub mod metrics;
pub mod state_key;

pub mod router;

mod position_manager;

pub use self::metrics::register_metrics;
pub use component::{Dex, StateReadExt, StateWriteExt};
pub use position_manager::{PositionManager, PositionRead};

#[cfg(test)]
mod tests;
