mod ciphertext;
mod plaintext;

use anyhow::Result;
use ark_ff::PrimeField;
pub use ciphertext::SwapCiphertext;
use decaf377::Fq;
pub use plaintext::SwapPlaintext;

use once_cell::sync::Lazy;

use penumbra_proto::{dex as pb, Protobuf};

// Swap ciphertext byte length
pub const SWAP_CIPHERTEXT_BYTES: usize = 216;
// Swap plaintext byte length
pub const SWAP_LEN_BYTES: usize = 200;

pub const OVK_WRAPPED_LEN_BYTES: usize = 80;

pub static DOMAIN_SEPARATOR: Lazy<Fq> =
    Lazy::new(|| Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.swap").as_bytes()));

#[derive(Clone, Debug, Copy)]
pub struct BatchSwapOutputData {
    pub delta_1: u64,
    pub delta_2: u64,
    pub lambda_1: u64,
    pub lambda_2: u64,
    pub success: bool,
}

impl Protobuf<pb::BatchSwapOutputData> for BatchSwapOutputData {}

impl From<BatchSwapOutputData> for pb::BatchSwapOutputData {
    fn from(s: BatchSwapOutputData) -> Self {
        pb::BatchSwapOutputData {
            delta_1: s.delta_1,
            delta_2: s.delta_2,
            lambda_1: s.lambda_1,
            lambda_2: s.lambda_2,
            success: s.success,
        }
    }
}

impl TryFrom<pb::BatchSwapOutputData> for BatchSwapOutputData {
    type Error = anyhow::Error;
    fn try_from(s: pb::BatchSwapOutputData) -> Result<Self, Self::Error> {
        Ok(Self {
            delta_1: s.delta_1,
            delta_2: s.delta_2,
            lambda_1: s.lambda_1,
            lambda_2: s.lambda_2,
            success: s.success,
        })
    }
}
