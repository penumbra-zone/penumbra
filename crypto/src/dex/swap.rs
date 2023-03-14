use anyhow::{anyhow, Result};
use ark_ff::PrimeField;

use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::{r1cs::FqVar, Fq};
use once_cell::sync::Lazy;
use penumbra_proto::{
    client::v1alpha1::BatchSwapOutputDataResponse, core::dex::v1alpha1 as pb, DomainType,
};

use crate::{fixpoint::U128x128, Amount};

use super::{trading_pair::TradingPairVar, TradingPair};

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
    pub delta_1: Amount,
    pub delta_2: Amount,
    pub lambda_1_1: Amount,
    pub lambda_2_1: Amount,
    pub lambda_1_2: Amount,
    pub lambda_2_2: Amount,
    pub height: u64,
    pub trading_pair: TradingPair,
}

impl BatchSwapOutputData {
    /// Given a user's inputs `(delta_1_i, delta_2_i)`, compute their pro rata share
    /// of the batch output `(lambda_1_i, lambda_2_i)`.
    pub fn pro_rata_outputs(&self, (delta_1_i, delta_2_i): (Amount, Amount)) -> (Amount, Amount) {
        // The pro rata fraction is delta_j_i / delta_j, which we can multiply through:
        //   lambda_2_i = (delta_1_i / delta_1) * lambda_2_1 + (delta_2_i / delta_2) * lambda_2_2
        //   lambda_1_i = (delta_1_i / delta_1) * lambda_1_1 + (delta_2_i / delta_2) * lambda_1_2
        // But we want to compute these as
        //   lambda_2_i = (delta_1_i * lambda_2_1) / delta_1 + (delta_2_i * lambda_2_2) / delta_2
        //   lambda_1_i = (delta_1_i * lambda_1_1) / delta_1 + (delta_2_i * lambda_1_2) / delta_2
        // so that we can do division and rounding at the end.
        let lambda_2_i: U128x128 = ((U128x128::from(delta_1_i) * U128x128::from(self.lambda_2_1))
            .unwrap_or(0u64.into())
            .checked_div(&U128x128::from(self.delta_1))
            .unwrap_or(0u64.into())
            + (U128x128::from(delta_2_i) * U128x128::from(self.lambda_2_2))
                .unwrap_or(0u64.into())
                .checked_div(&U128x128::from(self.delta_2))
                .unwrap_or(0u64.into()))
        .unwrap_or(0u64.into());
        let lambda_1_i: U128x128 = ((U128x128::from(delta_1_i) * U128x128::from(self.lambda_2_1))
            .unwrap_or(0u64.into())
            .checked_div(&U128x128::from(self.delta_1))
            .unwrap_or(0u64.into())
            + (U128x128::from(delta_2_i) * U128x128::from(self.lambda_1_2))
                .unwrap_or(0u64.into())
                .checked_div(&U128x128::from(self.delta_2))
                .unwrap_or(0u64.into()))
        .unwrap_or(0u64.into());

        (
            lambda_1_i.try_into().unwrap_or(0u64.into()),
            lambda_2_i.try_into().unwrap_or(0u64.into()),
        )
    }
}

pub struct BatchSwapOutputDataVar {
    pub trading_pair: TradingPairVar,
    pub height: FqVar,
}

impl AllocVar<BatchSwapOutputData, Fq> for BatchSwapOutputDataVar {
    fn new_variable<T: std::borrow::Borrow<BatchSwapOutputData>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let output_data = f()?.borrow().clone();
        let trading_pair =
            TradingPairVar::new_variable(cs.clone(), || Ok(output_data.trading_pair), mode)?;
        let height = FqVar::new_variable(cs.clone(), || Ok(Fq::from(output_data.height)), mode)?;
        Ok(Self {
            trading_pair,
            height,
        })
    }
}

impl DomainType for BatchSwapOutputData {
    type Proto = pb::BatchSwapOutputData;
}

impl From<BatchSwapOutputData> for pb::BatchSwapOutputData {
    fn from(s: BatchSwapOutputData) -> Self {
        pb::BatchSwapOutputData {
            delta_1: Some(s.delta_1.into()),
            delta_2: Some(s.delta_2.into()),
            trading_pair: Some(s.trading_pair.into()),
            height: s.height,
            lambda_1_1: Some(s.lambda_1_1.into()),
            lambda_2_1: Some(s.lambda_2_1.into()),
            lambda_1_2: Some(s.lambda_1_2.into()),
            lambda_2_2: Some(s.lambda_2_2.into()),
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
            delta_1: s
                .delta_1
                .ok_or_else(|| anyhow!("Missing delta_1"))?
                .try_into()?,
            delta_2: s
                .delta_2
                .ok_or_else(|| anyhow!("Missing delta_2"))?
                .try_into()?,
            lambda_1_1: s
                .lambda_1_1
                .ok_or_else(|| anyhow!("Missing lambda_1_1"))?
                .try_into()?,
            lambda_2_1: s
                .lambda_2_1
                .ok_or_else(|| anyhow!("Missing lambda_2_1"))?
                .try_into()?,
            lambda_1_2: s
                .lambda_1_2
                .ok_or_else(|| anyhow!("Missing lambda_1_2"))?
                .try_into()?,
            lambda_2_2: s
                .lambda_2_2
                .ok_or_else(|| anyhow!("Missing lambda_2_2"))?
                .try_into()?,
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
