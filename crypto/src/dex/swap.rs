use anyhow::{anyhow, Result};
use ark_ff::PrimeField;

use decaf377::Fq;
use once_cell::sync::Lazy;
use penumbra_proto::{
    client::v1alpha1::BatchSwapOutputDataResponse, core::dex::v1alpha1 as pb, DomainType,
};

use super::TradingPair;

mod ciphertext;
mod payload;
mod plaintext;

pub use ciphertext::SwapCiphertext;
pub use payload::SwapPayload;
pub use plaintext::{SwapPlaintext, SwapPlaintextVar};

// Swap ciphertext byte length.
pub const SWAP_CIPHERTEXT_BYTES: usize = 272;
// Swap plaintext byte length.
pub const SWAP_LEN_BYTES: usize = 256;

pub static DOMAIN_SEPARATOR: Lazy<Fq> =
    Lazy::new(|| Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.swap").as_bytes()));

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct BatchSwapOutputData {
    pub delta_1: u64,
    pub delta_2: u64,
    pub lambda_1_1: u64,
    pub lambda_2_1: u64,
    pub lambda_1_2: u64,
    pub lambda_2_2: u64,
    pub height: u64,
    pub trading_pair: TradingPair,
}

impl BatchSwapOutputData {
    /// Given a user's inputs `(delta_1_i, delta_2_i)`, compute their pro rata share
    /// of the batch output `(lambda_1_i, lambda_2_i)`.
    pub fn pro_rata_outputs(&self, (delta_1_i, delta_2_i): (u64, u64)) -> (u64, u64) {
        // The pro rata fraction is delta_j_i / delta_j, which we can multiply through:
        //   lambda_2_i = (delta_1_i / delta_1) * lambda_2_1 + (delta_2_i / delta_2) * lambda_2_2
        //   lambda_1_i = (delta_1_i / delta_1) * lambda_1_1 + (delta_2_i / delta_2) * lambda_1_2
        // But we want to compute these as
        //   lambda_2_i = (delta_1_i * lambda_2_1) / delta_1 + (delta_2_i * lambda_2_2) / delta_2
        //   lambda_1_i = (delta_1_i * lambda_1_1) / delta_1 + (delta_2_i * lambda_1_2) / delta_2
        // so that we can do division and rounding at the end.
        let lambda_2_i = ((delta_1_i as u128) * (self.lambda_2_1 as u128))
            .checked_div(self.delta_1 as u128)
            .unwrap_or(0)
            + ((delta_2_i as u128) * (self.lambda_2_2 as u128))
                .checked_div(self.delta_2 as u128)
                .unwrap_or(0);
        let lambda_1_i = ((delta_1_i as u128) * (self.lambda_2_1 as u128))
            .checked_div(self.delta_1 as u128)
            .unwrap_or(0)
            + ((delta_2_i as u128) * (self.lambda_1_2 as u128))
                .checked_div(self.delta_2 as u128)
                .unwrap_or(0);

        (lambda_1_i as u64, lambda_2_i as u64)
    }
}

impl DomainType for BatchSwapOutputData {
    type Proto = pb::BatchSwapOutputData;
}

impl From<BatchSwapOutputData> for pb::BatchSwapOutputData {
    fn from(s: BatchSwapOutputData) -> Self {
        pb::BatchSwapOutputData {
            delta_1: s.delta_1,
            delta_2: s.delta_2,
            trading_pair: Some(s.trading_pair.into()),
            height: s.height,
            lambda_1_1: s.lambda_1_1,
            lambda_2_1: s.lambda_2_1,
            lambda_1_2: s.lambda_1_2,
            lambda_2_2: s.lambda_2_2,
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
            lambda_1_1: s.lambda_1_1,
            lambda_2_1: s.lambda_2_1,
            lambda_1_2: s.lambda_1_2,
            lambda_2_2: s.lambda_2_2,
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
