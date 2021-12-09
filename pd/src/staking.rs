use num::bigint;
use num::rational;
use penumbra_crypto::{Address, Value};
use serde::{Deserialize, Serialize};
use tendermint::{vote, PublicKey};

const PENUMBRA_BECH32_VALIDATOR_PREFIX: &str = "penumbravalpub";
/// Validator tracks the Penumbra validator's long-term consensus key (tm_pubkey), as well as their
/// voting power.
#[derive(Deserialize, Serialize, Debug, Eq, Clone)]
pub struct Validator {
    /// tm_pubkey is the validator's long-term Tendermint public key, where the private component
    /// is used to sign blocks.
    tm_pubkey: PublicKey,

    /// voting_power is the validator's current voting power.
    pub voting_power: vote::Power,

    /// commission_address is the validator's shielded commission address, where they receive their
    /// portion of the staking rewards.
    #[serde(with = "serde_with::rust::display_fromstr")]
    pub commission_address: Address,

    /// commission_rate is the portion of staking rewards that go to the validator (as opposed to
    /// the delegators).
    pub commission_rate: rational::Ratio<bigint::BigInt>,

    /// unclaimed_reward is the amount of commission that the validator has yet to claim.
    pub unclaimed_reward: Value,
}

impl PartialEq for Validator {
    fn eq(&self, other: &Self) -> bool {
        self.tm_pubkey == other.tm_pubkey
    }
}

impl Validator {
    pub fn new(pubkey: PublicKey, voting_power: vote::Power) -> Validator {
        Validator {
            tm_pubkey: pubkey,
            voting_power,
        }
    }

    /// consensus_address returns the bech32-encoded address of the validator's primary consensus
    /// public key.
    ///
    /// TKTK: should this return an address type?
    pub fn consensus_address(&self) -> String {
        self.tm_pubkey.to_bech32(PENUMBRA_BECH32_VALIDATOR_PREFIX)
    }
}
