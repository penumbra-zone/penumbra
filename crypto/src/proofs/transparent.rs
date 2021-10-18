//! Transparent proofs for `MVP1` of the Penumbra system.
use crate::Fr;

pub struct SpendProof {
    _spend_auth_randomizer: Fr,
    // more TK
}

impl SpendProof {
    pub fn new(spend_auth_randomizer: Fr) -> Self {
        Self {
            _spend_auth_randomizer: spend_auth_randomizer,
        }
    }
}

// OutputProof
