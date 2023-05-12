//! The Penumbra shielded pool [`Component`] and [`ActionHandler`] implementations.

mod action_handler;
mod metrics;
mod note_manager;
mod shielded_pool;
mod supply;

pub use self::metrics::register_metrics;
pub use note_manager::{NoteManager, StatePayload};
pub use shielded_pool::{ShieldedPool, StateReadExt};
pub use supply::{SupplyRead, SupplyWrite};
