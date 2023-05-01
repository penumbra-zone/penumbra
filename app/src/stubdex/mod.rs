//! The stubdex component contains implementations of a stubbed CPMM with fixed genesis token supplies.
//! It will run in parallel along with the [crate::dex] component until the [crate::dex] component implementation is complete
//! and the [crate::dex] component can process [penumbra_transaction::Action::Swap] and [penumbra_transaction::Action::SwapClaim] actions.
mod component;
pub mod metrics;
pub mod state_key;
mod stub_cpmm;

use stub_cpmm::StubCpmm;

pub use self::metrics::register_metrics;
pub use component::{StateReadExt, StateWriteExt, StubDex};

#[cfg(test)]
mod tests;
