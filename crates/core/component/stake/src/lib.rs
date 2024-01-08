#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(clippy::unwrap_used)]
#![allow(clippy::clone_on_copy)]

mod changes;
mod current_consensus_keys;
mod event;
mod funding_stream;
mod uptime;

// TODO: move into component mod like other component crates
#[cfg(feature = "component")]
mod action_handler;

#[cfg(feature = "component")]
pub mod component;

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
pub use undelegate_claim::{
    UndelegateClaim, UndelegateClaimBody, UndelegateClaimPlan, UndelegateClaimProof,
};

mod delegation_token;
mod governance_key;
mod identity_key;
mod penalty;
mod unbonding_token;

pub use delegation_token::DelegationToken;
pub use governance_key::GovernanceKey;
pub use identity_key::IdentityKey;
pub use penalty::Penalty;
pub use unbonding_token::UnbondingToken;

pub use changes::DelegationChanges;
pub use current_consensus_keys::CurrentConsensusKeys;
pub use funding_stream::{FundingStream, FundingStreams};
pub use uptime::Uptime;

pub mod genesis;
pub mod params;
