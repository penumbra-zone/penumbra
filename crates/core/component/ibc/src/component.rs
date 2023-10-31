mod action_handler;
pub mod app_handler;
mod channel;
mod client;
mod client_counter;
mod connection;
mod connection_counter;

#[cfg(feature = "rpc")]
pub mod rpc;

mod ibc_component;
mod metrics;
mod msg_handler;
pub mod packet;
mod proof_verification;
pub mod state_key;
// TODO: move this to the shielded pool crate
// pub mod transfer;
mod view;

use msg_handler::MsgHandler;

pub use self::metrics::register_metrics;
pub use channel::StateReadExt as ChannelStateReadExt;
pub use client::StateReadExt as ClientStateReadExt;
pub use connection::StateReadExt as ConnectionStateReadExt;
pub use view::{StateReadExt, StateWriteExt};

pub use ibc_component::IBCComponent;
