use tendermint::{vote, PublicKey};

const PENUMBRA_BECH32_VALIDATOR_PREFIX: &str = "penumbravalpub";
/// Validator tracks the Penumbra validator's long-term consensus key (tm_pubkey), as well as their
/// voting power.
#[derive(Debug, Eq, PartialOrd, Ord)]
pub struct Validator {
    tm_pubkey: PublicKey,
    voting_power: vote::Power,
}

impl PartialEq for Validator {
    fn eq(&self, other: &Self) -> bool {
        self.tm_pubkey == other.tm_pubkey
    }
}

impl Validator {
    pub fn new(pubkey: PublicKey, voting_power: vote::Power) -> Validator {
        pubkey.to_bech32(PENUMBRA_BECH32_VALIDATOR_PREFIX);
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
