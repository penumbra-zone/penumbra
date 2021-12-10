use penumbra_crypto::Address;
use serde::{Deserialize, Serialize};
use tendermint::{vote, PublicKey};

const PENUMBRA_BECH32_VALIDATOR_PREFIX: &str = "penumbravalpub";
/// Validator tracks the Penumbra validator's long-term consensus key (tm_pubkey), as well as their
/// voting power.
#[derive(Deserialize, Serialize, Debug, Eq, Clone)]
pub struct Validator {
    /// The validator's long-term Tendermint public key, where the private component
    /// is used to sign blocks.
    tm_pubkey: PublicKey,

    /// The validator's current voting power.
    pub voting_power: vote::Power,

    /// The validator's shielded commission address, where they receive their portion of the
    /// staking rewards.
    pub commission_address: Address,

    /// The portion of staking rewards that go to the validator (as opposed to the delegators).
    pub commission_rate_bps: u16,
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
        commission_address: Address,
        commission_rate_bps: u16,
    ) -> Validator {
        Validator {
            tm_pubkey: pubkey,
            voting_power,
            commission_address,
            commission_rate_bps,
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
