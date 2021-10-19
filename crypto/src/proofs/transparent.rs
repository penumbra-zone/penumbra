//! Transparent proofs for `MVP1` of the Penumbra system.

use ark_ff::Zero;
use ark_serialize::CanonicalSerialize;

use crate::merkle;
use crate::Fr;

pub const OUTPUT_PROOF_LEN_BYTES: usize = 192;
// xx check the spend proof len
pub const SPEND_PROOF_LEN_BYTES: usize = 192;

pub struct SpendProof {
    pub spend_auth_randomizer: Fr,
    pub merkle_path: merkle::Path,
    // more TK
}

impl Into<[u8; SPEND_PROOF_LEN_BYTES]> for SpendProof {
    fn into(self) -> [u8; SPEND_PROOF_LEN_BYTES] {
        let mut spend_auth_randomizer_bytes = [0u8; 32];
        self.spend_auth_randomizer
            .serialize(&mut spend_auth_randomizer_bytes[..])
            .expect("serialization into array should be infallible");

        let mut bytes = [0u8; SPEND_PROOF_LEN_BYTES];
        bytes.copy_from_slice(&spend_auth_randomizer_bytes);

        // TODO: Merkle path serialization and add in here

        // When we put more stuff into this transparent spend proof, add here.
        bytes
    }
}

impl From<&[u8]> for SpendProof {
    fn from(_raw_proof: &[u8]) -> SpendProof {
        // let bytes: [u8; SPEND_PROOF_LEN_BYTES] = ...

        // TODO!
        SpendProof {
            spend_auth_randomizer: Fr::zero(),
            merkle_path: merkle::Path::default(),
        }
    }
}

pub struct OutputProof {}

impl Into<[u8; OUTPUT_PROOF_LEN_BYTES]> for OutputProof {
    fn into(self) -> [u8; OUTPUT_PROOF_LEN_BYTES] {
        let bytes = [0u8; OUTPUT_PROOF_LEN_BYTES];
        // When we put more stuff into this transparent output proof, add here.
        bytes
    }
}
