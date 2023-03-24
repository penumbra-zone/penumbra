use anyhow::{anyhow, Result};
use ark_ff::{PrimeField, ToConstraintField};

use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::{r1cs::FqVar, Fq};
use once_cell::sync::Lazy;
use penumbra_proto::{
    client::v1alpha1::BatchSwapOutputDataResponse, core::dex::v1alpha1 as pb, DomainType,
};

use crate::{asset::AmountVar, fixpoint::U128x128, Amount};

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

impl ToConstraintField<Fq> for BatchSwapOutputData {
    fn to_field_elements(&self) -> Option<Vec<Fq>> {
        let mut public_inputs = Vec::new();
        public_inputs.extend(self.trading_pair.asset_1.0.to_field_elements().unwrap());
        public_inputs.extend(self.trading_pair.asset_2.0.to_field_elements().unwrap());
        public_inputs.extend(Fq::from(self.height).to_field_elements().unwrap());
        // public_inputs.extend(Fq::from(self.delta_1).to_field_elements().unwrap());
        // public_inputs.extend(Fq::from(self.delta_2).to_field_elements().unwrap());
        // public_inputs.extend(Fq::from(self.lambda_1_1).to_field_elements().unwrap());
        // public_inputs.extend(Fq::from(self.lambda_2_1).to_field_elements().unwrap());
        // public_inputs.extend(Fq::from(self.lambda_1_2).to_field_elements().unwrap());
        // public_inputs.extend(Fq::from(self.lambda_2_2).to_field_elements().unwrap());
        Some(public_inputs)
    }
}

pub struct BatchSwapOutputDataVar {
    pub trading_pair: TradingPairVar,
    pub height: FqVar,
    // pub delta_1: AmountVar,
    // pub delta_2: AmountVar,
    // pub lambda_1_1: AmountVar,
    // pub lambda_2_1: AmountVar,
    // pub lambda_1_2: AmountVar,
    // pub lambda_2_2: AmountVar,
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
        // let delta_1 =
        //     AmountVar::new_variable(cs.clone(), || Ok(Amount::from(output_data.delta_1)), mode)?;
        // let delta_2 =
        //     AmountVar::new_variable(cs.clone(), || Ok(Amount::from(output_data.delta_2)), mode)?;
        // let lambda_1_1 =
        //     AmountVar::new_variable(cs.clone(), || Ok(Amount::from(output_data.lambda_1_1)), mode)?;
        // let lambda_2_1 =
        //     AmountVar::new_variable(cs, || Ok(Amount::from(output_data.lambda_2_1)), mode)?;
        // let lambda_1_2 =
        //     AmountVar::new_variable(cs.clone(), || Ok(Amount::from(output_data.lambda_1_2)), mode)?;
        // let lambda_2_2 =
        //     AmountVar::new_variable(cs, || Ok(Amount::from(output_data.lambda_2_2)), mode)?;
        Ok(Self {
            trading_pair,
            height,
            // delta_1,
            // delta_2,
            // lambda_1_1,
            // lambda_2_1,
            // lambda_1_2,
            // lambda_2_2
        })
    }
}

// impl BatchSwapOutputDataVar {
//     pub fn pro_rata_outputs(
//         &self,
//         delta_1_i: AmountVar,
//         delta_2_i: AmountVar,
//     ) -> Result<(AmountVar, AmountVar), SynthesisError> {
//         // let lambda_2_i = ((delta_1_i as u128) * (self.lambda_2 as u128))
//         //     .checked_div(self.delta_1 as u128)
//         //     .unwrap_or(0);
//         let numerator_2_var = delta_1_i.clone() * self.lambda_2.clone();
//         let (lambda_2_i, _) = numerator_2_var.quo_rem(&self.delta_1)?;

//         // let lambda_1_i = ((delta_2_i as u128) * (self.lambda_1 as u128))
//         //     .checked_div(self.delta_2 as u128)
//         //     .unwrap_or(0);
//         let numerator_1_var = delta_2_i.clone() * self.lambda_1.clone();
//         let (lambda_1_i, _) = numerator_1_var.quo_rem(&self.delta_2)?;

//         // If success we return the results of the above computation ((lambda_1_i, lambda_2_i a)
//         // Else we return (delta_1_i, delta_2_i)
//         // let return_var_1 = AmountVar::conditionally_select(&self.success, &lambda_1_i, &delta_1_i)?;
//         // let return_var_2 = AmountVar::conditionally_select(&self.success, &lambda_2_i, &delta_2_i)?;
//         // Ok((return_var_1, return_var_2))
//         Ok((lambda_1_i, lambda_2_i))
//     }
// }

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
