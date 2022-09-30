// Many of the IBC message types are enums, where the number of variants differs
// depending on the build configuration, meaning that the fallthrough case gets
// marked as unreachable only when not building in test configuration.
#![allow(unreachable_patterns)]

mod client;
mod component;
mod connection;
pub(crate) mod event;
mod ibc_token;
mod packet;
pub use ibc_token::IBCToken;
mod ibc_handler;
mod metrics;
mod transfer;

pub use self::metrics::register_metrics;

pub use client::{
    validate_penumbra_client_state, ClientConnections, ClientCounter, VerifiedHeights,
};
pub use component::IBCComponent;
pub use connection::{ConnectionCounter, SUPPORTED_VERSIONS};
