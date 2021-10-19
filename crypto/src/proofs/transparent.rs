//! Transparent proofs for `MVP1` of the Penumbra system.
use crate::merkle;
use crate::Fr;

pub const OUTPUT_PROOF_LEN_BYTES: usize = 192;

pub struct SpendProof {
    pub spend_auth_randomizer: Fr,
    pub merkle_path: merkle::Path,
    // more TK
}

pub struct OutputProof {}

impl Into<[u8; OUTPUT_PROOF_LEN_BYTES]> for OutputProof {
    fn into(self) -> [u8; OUTPUT_PROOF_LEN_BYTES] {
        let mut bytes = [0u8; OUTPUT_PROOF_LEN_BYTES];
        // When we put more stuff into this transparent output proof, add here.
        bytes
    }
}
