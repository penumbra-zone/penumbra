// Many of the IBC message types are enums, where the number of variants differs
// depending on the build configuration, meaning that the fallthrough case gets
// marked as unreachable only when not building in test configuration.
#![deny(clippy::unwrap_used)]
#![allow(unreachable_patterns)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;

pub mod genesis;
mod ibc_action;
mod ibc_token;
mod ics20_withdrawal;
pub mod params;
mod version;

pub use ibc_action::IbcAction;
pub use ibc_action::IbcActionWithHandler;
pub use ibc_token::IbcToken;
pub use ics20_withdrawal::Ics20Withdrawal;

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub use component::{StateReadExt, StateWriteExt};
