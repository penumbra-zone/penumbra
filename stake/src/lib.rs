mod epoch;
mod funding_stream;
mod rate;
mod token;
mod validator;

pub use epoch::Epoch;
pub use funding_stream::FundingStream;
pub use rate::{BaseRateData, RateData};
pub use token::DelegationToken;
pub use validator::Validator;

/// The Bech32 prefix used for validator identity keys.
pub const VALIDATOR_IDENTITY_BECH32_PREFIX: &str = "penumbravalid";

/// The Bech32 prefix used for validator consensus pubkeys.
pub const VALIDATOR_CONSENSUS_BECH32_PREFIX: &str = "penumbravalconspub";
