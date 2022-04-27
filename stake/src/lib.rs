#![allow(clippy::clone_on_copy)]
use once_cell::sync::Lazy;
use penumbra_crypto::asset;
use penumbra_crypto::IdentityKey;

mod changes;
mod commission;
mod epoch;
mod funding_stream;
mod token;
mod uptime;

pub mod action;
pub mod rate;
pub mod validator;

pub use changes::DelegationChanges;
pub use commission::{CommissionAmount, CommissionAmounts};
pub use epoch::Epoch;
pub use funding_stream::{FundingStream, FundingStreams};
pub use token::DelegationToken;
pub use uptime::Uptime;

/// The Bech32 prefix used for validator consensus pubkeys.
//pub const VALIDATOR_CONSENSUS_BECH32_PREFIX: &str = "penumbravalconspub";

// TODO: go through the source tree and use these instead of hardcoding "upenumbra"

pub static STAKING_TOKEN_DENOM: Lazy<asset::Denom> =
    Lazy::new(|| asset::REGISTRY.parse_denom("upenumbra").unwrap());
pub static STAKING_TOKEN_ASSET_ID: Lazy<asset::Id> = Lazy::new(|| STAKING_TOKEN_DENOM.id());
