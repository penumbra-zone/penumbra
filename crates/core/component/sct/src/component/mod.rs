// The SCT component implementation.
pub mod sct;
// Blockchain clocks: epoch indices, block heights and timestamps.
pub mod clock;
// Implementation of the SCT component query server.
pub mod rpc;
// Tracking commitment sources within a block execution.
pub mod source;
// Mediate access to the state commitment tree and related data.
pub mod tree;

// Access to configuration data for the component.
pub use sct::{StateReadExt, StateWriteExt};

// Note: for some reason, `rust-analyzer` chokes when this file is named
// `component.rs`. If you read this and manage to fix it, please rename it.
