mod ciphertext;
mod plaintext;

use anyhow::{anyhow, Result};
use ark_ff::PrimeField;
use blake2b_simd::Hash;
pub use ciphertext::SwapCiphertext;
use decaf377::Fq;
pub use plaintext::SwapPlaintext;

use once_cell::sync::Lazy;

use penumbra_proto::{dex as pb, Protobuf};

use super::TradingPair;

// Swap ciphertext byte length
pub const SWAP_CIPHERTEXT_BYTES: usize = 216;
// Swap plaintext byte length
pub const SWAP_LEN_BYTES: usize = 200;

pub const OVK_WRAPPED_LEN_BYTES: usize = 80;

pub static DOMAIN_SEPARATOR: Lazy<Fq> =
    Lazy::new(|| Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.swap").as_bytes()));

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct BatchSwapOutputData {
    pub delta_1: u64,
    pub delta_2: u64,
    pub lambda_1: u64,
    pub lambda_2: u64,
    pub height: u64,
    pub trading_pair: TradingPair,
    pub success: bool,
}

impl BatchSwapOutputData {
    pub fn auth_hash(&self) -> Hash {
        blake2b_simd::Params::default()
            .personal(b"PAH:batch_swap_output_data")
            .to_state()
            .update(&self.delta_1.to_le_bytes())
            .update(&self.delta_2.to_le_bytes())
            .update(&self.lambda_1.to_le_bytes())
            .update(&self.lambda_2.to_le_bytes())
            .update(&self.height.to_le_bytes())
            .update(self.trading_pair.auth_hash().as_bytes())
            .update(&(self.success as i64).to_le_bytes())
            .finalize()
    }
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
            trading_pair: Some(s.trading_pair.into()),
            height: s.height,
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
            height: s.height,
            trading_pair: s
                .trading_pair
                .ok_or_else(|| anyhow!("Missing trading_pair"))?
                .try_into()?,
        })
    }
}
