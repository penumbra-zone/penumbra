//! The dex component contains implementations of the real Penumbra CPMM with token supplies based on liquidity provider interactions.
//! It will run in parallel along with the [swapdex] component until the dex component implementation is complete
//! and the dex component can process [penumbra_transaction::Action::Swap] and [penumbra_transaction::Action::SwapClaim] actions.
mod component;
pub mod metrics;
pub mod state_key;

mod position_manager;

pub use self::metrics::register_metrics;
pub use component::{Dex, StateReadExt, StateWriteExt};
pub use position_manager::{PositionManager, PositionRead};

#[cfg(test)]
mod tests;
