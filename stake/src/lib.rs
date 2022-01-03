mod epoch;
mod funding_stream;
mod identity_key;
mod rate;
mod token;
mod validator;

pub use epoch::Epoch;
pub use funding_stream::FundingStream;
pub use identity_key::IdentityKey;
pub use rate::{BaseRateData, RateData};
pub use token::DelegationToken;
pub use validator::Validator;

/// The Bech32 prefix used for validator consensus pubkeys.
pub const VALIDATOR_CONSENSUS_BECH32_PREFIX: &str = "penumbravalconspub";

pub use penumbra_proto::serializers::bech32str::validator_identity_key::BECH32_PREFIX as VALIDATOR_IDENTITY_BECH32_PREFIX;
