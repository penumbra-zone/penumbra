// Many of the IBC message types are enums, where the number of variants differs
// depending on the build configuration, meaning that the fallthrough case gets
// marked as unreachable only when not building in test configuration.
#![deny(clippy::unwrap_used)]
#![allow(unreachable_patterns)]
// Requires nightly
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "component")]
pub mod component;
#[cfg(feature = "component")]
pub use component::ibc_action_with_handler::IbcRelayWithHandlers;

pub mod genesis;
mod ibc_action;
mod ibc_token;
pub mod params;
mod version;

mod prefix;
pub use prefix::{MerklePrefixExt, IBC_COMMITMENT_PREFIX, IBC_PROOF_SPECS, IBC_SUBSTORE_PREFIX};

pub use ibc_action::IbcRelay;
pub use ibc_token::IbcToken;

#[cfg(feature = "component")]
pub use component::{StateReadExt, StateWriteExt};
