use anyhow::{anyhow, Result};
use ark_ff::PrimeField;
use blake2b_simd::Hash;
use decaf377::Fq;
use once_cell::sync::Lazy;
use penumbra_proto::{
    client::v1alpha1::BatchSwapOutputDataResponse, core::dex::v1alpha1 as pb, Protobuf,
};

use super::TradingPair;

mod ciphertext;
mod payload;
mod plaintext;

pub use ciphertext::SwapCiphertext;
pub use payload::SwapPayload;
pub use plaintext::SwapPlaintext;

// Swap ciphertext byte length
pub const SWAP_CIPHERTEXT_BYTES: usize = 248;
// Swap plaintext byte length
pub const SWAP_LEN_BYTES: usize = 232;

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
    /// Given a user's inputs `(delta_1_i, delta_2_i)`, compute their pro rata share
    /// of the batch output `(lambda_1_i, lambda_2_i)`.
    pub fn pro_rata_outputs(&self, (delta_1_i, delta_2_i): (u64, u64)) -> (u64, u64) {
        if self.success {
            // The swap succeeded, so the pro rata share is a share of the output amount of
            // the opposite token type.
            // The pro rata fraction is delta_j_i / delta_j, which we can multiply through:
            //   lambda_2_i = (delta_1_i / delta_1) * lambda_2
            //   lambda_1_i = (delta_2_i / delta_2) * lambda_1
            // But we want to compute these as
            //   lambda_2_i = (delta_1_i * lambda_2) / delta_1
            //   lambda_1_i = (delta_2_i * lambda_1) / delta_2
            // so that we can do division and rounding at the end.
            let lambda_2_i = ((delta_1_i as u128) * (self.lambda_2 as u128))
                .checked_div(self.delta_1 as u128)
                .unwrap_or(0);
            let lambda_1_i = ((delta_2_i as u128) * (self.lambda_1 as u128))
                .checked_div(self.delta_2 as u128)
                .unwrap_or(0);

            (lambda_1_i as u64, lambda_2_i as u64)
        } else {
            // The swap failed, so the pro rata share is a share of the input amount of
            // the same token type. But this is exactly the delta_j_i.
            (delta_1_i, delta_2_i)
        }
    }

    pub fn auth_hash(&self) -> Hash {
        blake2b_simd::Params::default()
            .personal(b"PAH:btchswp_otpt")
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

impl From<BatchSwapOutputData> for BatchSwapOutputDataResponse {
    fn from(s: BatchSwapOutputData) -> Self {
        BatchSwapOutputDataResponse {
            data: Some(s.into()),
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

impl TryFrom<BatchSwapOutputDataResponse> for BatchSwapOutputData {
    type Error = anyhow::Error;
    fn try_from(value: BatchSwapOutputDataResponse) -> Result<Self, Self::Error> {
        value
            .data
            .ok_or_else(|| anyhow::anyhow!("empty BatchSwapOutputDataResponse message"))?
            .try_into()
    }
}
