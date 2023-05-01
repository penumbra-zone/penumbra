#[macro_use]
extern crate serde_with;

mod action_handler;
mod mock_client;
mod temp_storage_ext;

pub use action_handler::ActionHandler;
pub use mock_client::MockClient;
pub use temp_storage_ext::TempStorageExt;

pub mod app;
pub mod governance;
pub mod stake;
pub mod stubdex;

/// Temporary compat wrapper for duplicate trait impls
pub struct Compat<'a, T>(&'a T);
