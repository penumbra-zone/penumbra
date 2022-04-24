// Many of the IBC message types are enums, where the number of variants differs
// depending on the build configuration, meaning that the fallthrough case gets
// marked as unreachable only when not building in test configuration.
#![allow(unreachable_patterns)]

mod client;
mod ibcaction;

pub use client::{ClientCounter, ClientData, ConsensusState, VerifiedHeights};
pub use ibcaction::IBCAction;
