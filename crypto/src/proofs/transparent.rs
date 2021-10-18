//! Transparent proofs for `MVP1` of the Penumbra system.
use crate::merkle;
use crate::Fr;

pub struct SpendProof {
    pub spend_auth_randomizer: Fr,
    pub merkle_path: merkle::Path,
    // more TK
}

pub struct OutputProof {}
