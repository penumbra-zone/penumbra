//! The Sct component contains the interface to the chain's state commitment tree
//! and nullifier set. It also serves as tracking the various chain clocks, whether
//! logical, like an epoch index, or a block height, or physical, like block timestamps.

/// Blockchain clocks: epoch indices, block heights and timestamps.
pub mod clock;
/// Implementation of the SCT component query server.
pub mod rpc;
/// The SCT component implementation.
pub mod sct;
/// Tracking commitment sources within a block execution.
pub mod source;
/// Mediate access to the state commitment tree and related data.
pub mod tree;

// Access to configuration data for the component.
pub use sct::{StateReadExt, StateWriteExt};

// Note: for some reason, `rust-analyzer` chokes when this file is named
// `component.rs`. If you read this and manage to fix it, please rename it.
