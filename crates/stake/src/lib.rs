#![allow(clippy::clone_on_copy)]

use penumbra_component::ActionHandler;
use penumbra_crypto::stake::IdentityKey;

mod action_handler;
mod changes;
mod current_consensus_keys;
mod event;
mod funding_stream;
mod metrics;
mod uptime;

pub mod component;
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
pub use component::StateReadExt;
pub use current_consensus_keys::CurrentConsensusKeys;
pub use funding_stream::{FundingStream, FundingStreams};
pub use uptime::Uptime;
