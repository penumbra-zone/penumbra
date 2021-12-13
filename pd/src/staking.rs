use penumbra_crypto::Address;
use serde::{Deserialize, Serialize};
use tendermint::{vote, PublicKey};

const PENUMBRA_BECH32_VALIDATOR_PREFIX: &str = "penumbravalpub";

/// A destination for a portion of a validator's commission of staking rewards.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct FundingStream {
    /// The destinatination address for the funding stream..
    pub address: Address,

    /// The portion (in terms of [basis points](https://en.wikipedia.org/wiki/Basis_point)) of the
    /// validator's total staking reward that goes to this funding stream.
    pub rate_bps: u16,
}
/// Validator tracks the Penumbra validator's long-term consensus key (tm_pubkey), as well as their
/// voting power.
#[derive(Deserialize, Serialize, Debug, Eq, Clone)]
pub struct Validator {
    /// The validator's long-term Tendermint public key, where the private component
    /// is used to sign blocks.
    tm_pubkey: PublicKey,

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
        self.tm_pubkey.to_bech32(PENUMBRA_BECH32_VALIDATOR_PREFIX)
    }

    // why isn't tm_pubkey public?
    pub fn tm_pubkey(&self) -> &PublicKey {
        &self.tm_pubkey
    }
}
