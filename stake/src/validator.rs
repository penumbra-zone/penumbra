use serde::{Deserialize, Serialize};
use tendermint::{vote, PublicKey};

use crate::{FundingStream, VALIDATOR_IDENTITY_BECH32_PREFIX};

/// Validator tracks the Penumbra validator's long-term consensus key (tm_pubkey), as well as their
/// voting power.
#[derive(Deserialize, Serialize, Debug, Eq, Clone)]
pub struct Validator {
    /// The validator's long-term Tendermint public key, where the private component
    /// is used to sign blocks.
    pub tm_pubkey: PublicKey,

    /// The validator's current voting power.
    pub voting_power: vote::Power,

    /// The destinations for the validator's staking reward. The commission is implicitly defined
    /// by the configuration of funding_streams, the sum of FundingStream.rate_bps.
    ///
    /// NOTE: sum(FundingRate.rate_bps) should not exceed 100% (10000bps. For now, we ignore this
    /// condition, in the future we should probably make it a slashable offense.
    pub funding_streams: Vec<FundingStream>,
    // NOTE: unclaimed rewards are tracked by inserting reward notes for the last epoch into the
    // NCT at the beginning of each epoch
}

impl PartialEq for Validator {
    fn eq(&self, other: &Self) -> bool {
        self.tm_pubkey == other.tm_pubkey
    }
}

impl Validator {
    pub fn new(
        pubkey: PublicKey,
        voting_power: vote::Power,
        funding_streams: Vec<FundingStream>,
    ) -> Validator {
        Validator {
            tm_pubkey: pubkey,
            voting_power,
            funding_streams,
        }
    }

    /// consensus_address returns the bech32-encoded address of the validator's primary consensus
    /// public key.
    ///
    /// TKTK: should this return an address type?
    pub fn consensus_address(&self) -> String {
        self.tm_pubkey.to_bech32(VALIDATOR_IDENTITY_BECH32_PREFIX)
    }

    /// compute the validator's commission rate by summing its funding streams.
    ///
    ///
    pub fn commission_rate(&self) -> u16 {
        self.funding_streams
            .iter()
            .fold(0 as u16, |sum, fs| sum.checked_add(fs.rate_bps).unwrap())
    }
}
