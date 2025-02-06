// Requires nightly
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(clippy::unwrap_used)]
#![allow(clippy::clone_on_copy)]

mod changes;
mod current_consensus_keys;
mod delegation_token;
pub mod event;
mod governance_key;
mod identity_key;
mod penalty;
mod unbonding_token;
mod uptime;

pub mod delegate;
pub mod funding_stream;
pub mod genesis;
pub mod params;
pub mod rate;
pub mod state_key;
pub mod undelegate;
pub mod undelegate_claim;
pub mod validator;

#[cfg(feature = "component")]
pub mod component;

#[cfg(feature = "component")]
pub use component::{StateReadExt, StateWriteExt};

pub static BPS_SQUARED_SCALING_FACTOR: once_cell::sync::Lazy<penumbra_sdk_num::fixpoint::U128x128> =
    once_cell::sync::Lazy::new(|| 1_0000_0000u128.into());

pub use self::delegate::Delegate;
pub use self::undelegate::Undelegate;
pub use self::undelegate_claim::{
    UndelegateClaim, UndelegateClaimBody, UndelegateClaimPlan, UndelegateClaimProof,
};

pub use self::delegation_token::DelegationToken;
pub use self::governance_key::GovernanceKey;
pub use self::identity_key::IdentityKey;
pub use self::identity_key::IDENTITY_KEY_LEN_BYTES;
pub use self::penalty::Penalty;
pub use self::unbonding_token::UnbondingToken;

pub use self::changes::DelegationChanges;
pub use self::current_consensus_keys::CurrentConsensusKeys;
pub use self::funding_stream::{FundingStream, FundingStreams};
pub use self::uptime::Uptime;
