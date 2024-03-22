//! The Penumbra shielded pool [`Component`] and [`ActionHandler`] implementations.

mod action_handler;
mod assets;
mod metrics;
mod note_manager;
mod shielded_pool;
mod transfer;

pub use self::metrics::register_metrics;
pub use assets::{AssetRegistry, AssetRegistryRead};
pub use note_manager::NoteManager;
pub use shielded_pool::{ShieldedPool, StateReadExt, StateWriteExt};
pub use transfer::Ics20Transfer;

pub mod rpc;
