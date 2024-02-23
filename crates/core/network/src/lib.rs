//! Network configuration for Penumbra.
//!
//! Includes types for fullnodes and validators, as well as genesis,
//! allocations, and wrangling files for both `pd` and `cometbft`.

pub mod cometbft;
pub mod config;
pub mod fullnode;
pub mod generate;
pub mod join;
pub mod validator;
