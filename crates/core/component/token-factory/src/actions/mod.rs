//! Token factory actions.
//!
//! This module contains the actions for the token factory:
//!
//! - [`ActionTokenFactoryCreate`] - Create a new token, optionally with mint capability.
//! - [`ActionTokenFactoryMint`] - Mint additional tokens using the mint capability.

mod create;
mod mint;

pub use create::ActionTokenFactoryCreate;
pub use mint::ActionTokenFactoryMint;
