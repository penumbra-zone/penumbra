//! The Penumbra shielded pool [`Component`] and [`ActionHandler`] implementations.

mod action_handler;
mod metrics;
mod note_manager;
mod shielded_pool;
mod supply;
mod transfer;

pub mod ics20_withdrawal;
pub use ics20_withdrawal::Ics20Withdrawal;

pub use self::metrics::register_metrics;
pub use note_manager::NoteManager;
pub use shielded_pool::{ShieldedPool, StateReadExt};
pub use supply::{SupplyRead, SupplyWrite};
pub use transfer::Ics20Transfer;

pub mod rpc;
