#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::clone_on_copy)]

use penumbra_crypto::stake::IdentityKey;

mod changes;
mod current_consensus_keys;
mod event;
mod funding_stream;
mod metrics;
mod uptime;

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
mod action_handler;

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub use component::StateReadExt;

pub mod delegate;
pub mod rate;
pub mod state_key;
pub mod undelegate;
pub mod undelegate_claim;
pub mod validator;

pub use delegate::Delegate;
pub use undelegate::Undelegate;
pub use undelegate_claim::{UndelegateClaim, UndelegateClaimBody, UndelegateClaimPlan};

pub use self::metrics::register_metrics;
pub use changes::DelegationChanges;
pub use current_consensus_keys::CurrentConsensusKeys;
pub use funding_stream::{FundingStream, FundingStreams};
pub use uptime::Uptime;
