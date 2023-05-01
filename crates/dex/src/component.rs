//! The Penumbra DEX [`Component`] and [`ActionHandler`] implementations.

mod action_handler;
mod component;
mod metrics;
mod position_manager;
mod router;

pub use self::metrics::register_metrics;

pub use component::{Dex, StateReadExt, StateWriteExt};
pub use position_manager::{PositionManager, PositionRead};

#[cfg(test)]
mod tests;
