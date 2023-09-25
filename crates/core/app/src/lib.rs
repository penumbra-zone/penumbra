#![deny(clippy::unwrap_used)]
mod action_handler;
mod dao_ext;
mod mock_client;
mod temp_storage_ext;

pub use action_handler::ActionHandler;
pub use dao_ext::DaoStateReadExt;
pub use mock_client::MockClient;
pub use temp_storage_ext::TempStorageExt;

pub mod app;

pub mod metrics;
pub mod rpc;
pub use self::metrics::register_metrics;

#[cfg(test)]
mod tests;

/// Temporary compat wrapper for duplicate trait impls
pub struct Compat<'a, T>(&'a T);
