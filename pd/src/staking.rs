use tendermint::{vote, PublicKey};

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
        Validator{
            tm_pubkey: pubkey,
            voting_power,
        }
    }
    /*
    fn consensus_address() -> Address {
        // todo - bech32 encoding of tm_pubkey?
    }
    */
}