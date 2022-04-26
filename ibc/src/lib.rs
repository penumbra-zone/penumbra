// Many of the IBC message types are enums, where the number of variants differs
// depending on the build configuration, meaning that the fallthrough case gets
// marked as unreachable only when not building in test configuration.
#![allow(unreachable_patterns)]

mod client;
mod connection;
mod ibcaction;

pub use client::{ClientConnections, ClientCounter, ClientData, ConsensusState, VerifiedHeights};
pub use connection::{Connection, ConnectionCounter};
pub use ibcaction::IBCAction;
