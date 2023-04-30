/// The DAO isn't really a "component" because it doesn't really execute anything by itself. It's
/// just a collection of state that is modified by DaoSpend and DaoDeposit actions.
pub mod state_key;

mod action_handler;
mod view;

pub use view::{StateReadExt, StateWriteExt};
