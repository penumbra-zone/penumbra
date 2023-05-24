use anyhow::{anyhow, Result};

use ark_ff::ToConstraintField;
use ark_r1cs_std::{
    prelude::{AllocVar, EqGadget},
    select::CondSelectGadget,
};
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use decaf377::{r1cs::FqVar, Fq};
use penumbra_proto::{
    client::v1alpha1::BatchSwapOutputDataResponse, core::dex::v1alpha1 as pb, DomainType, TypeUrl,
};
use serde::{Deserialize, Serialize};

use penumbra_crypto::{
    asset::AmountVar,
    fixpoint::{U128x128, U128x128Var},
    Amount,
};

use crate::TradingPairVar;

use super::TradingPair;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::BatchSwapOutputData", into = "pb::BatchSwapOutputData")]
pub struct BatchSwapOutputData {
    /// The total amount of asset 1 that was input to the batch swap.
    pub delta_1: Amount,
    /// The total amount of asset 2 that was input to the batch swap.
    pub delta_2: Amount,
    /// The total amount of asset 1 that was output from the batch swap for 2=>1 trades.
    pub lambda_1: Amount,
    /// The total amount of asset 2 that was output from the batch swap for 1=>2 trades.
    pub lambda_2: Amount,
    /// The total amount of asset 1 that was returned unfilled from the batch swap for 1=>2 trades.
    pub unfilled_1: Amount,
    /// The total amount of asset 2 that was returned unfilled from the batch swap for 2=>1 trades.
    pub unfilled_2: Amount,
    /// The height for which the batch swap data is valid.
    pub height: u64,
    /// The trading pair associated with the batch swap.
    pub trading_pair: TradingPair,
    /// The starting block height of the epoch for which the batch swap data is valid.
    pub epoch_height: u64,
}

impl BatchSwapOutputData {
    /// Given a user's inputs `(delta_1_i, delta_2_i)`, compute their pro rata share
    /// of the batch output `(lambda_1_i, lambda_2_i)`.
    pub fn pro_rata_outputs(&self, (delta_1_i, delta_2_i): (Amount, Amount)) -> (Amount, Amount) {
        // The pro rata fraction is delta_j_i / delta_j, which we can multiply through:
        //   lambda_2_i = (delta_1_i / delta_1) * lambda_2   + (delta_2_i / delta_2) * unfilled_2
        //   lambda_1_i = (delta_1_i / delta_1) * unfilled_1 + (delta_2_i / delta_2) * lambda_1

        let delta_1_i = U128x128::from(delta_1_i);
        let delta_2_i = U128x128::from(delta_2_i);
        let delta_1 = U128x128::from(self.delta_1);
        let delta_2 = U128x128::from(self.delta_2);
        let lambda_1 = U128x128::from(self.lambda_1);
        let lambda_2 = U128x128::from(self.lambda_2);
        let unfilled_1 = U128x128::from(self.unfilled_1);
        let unfilled_2 = U128x128::from(self.unfilled_2);

        // Compute the user i's share of the batch inputs of assets 1 and 2.
        // The .unwrap_or_default ensures that when the batch input delta_1 is zero, all pro-rata shares of it are also zero.
        let pro_rata_input_1 = (delta_1_i / delta_1).unwrap_or_default();
        let pro_rata_input_2 = (delta_2_i / delta_2).unwrap_or_default();

        let lambda_2_i = (pro_rata_input_1 * lambda_2).unwrap_or_default()
            + (pro_rata_input_2 * unfilled_2).unwrap_or_default();
        let lambda_1_i = (pro_rata_input_1 * unfilled_1).unwrap_or_default()
            + (pro_rata_input_2 * lambda_1).unwrap_or_default();

        (
            lambda_1_i
                .unwrap_or_default()
                .round_down()
                .try_into()
                .expect("rounded amount is integral"),
            lambda_2_i
                .unwrap_or_default()
                .round_down()
                .try_into()
                .expect("rounded amount is integral"),
        )
    }
}

impl ToConstraintField<Fq> for BatchSwapOutputData {
    fn to_field_elements(&self) -> Option<Vec<Fq>> {
        let mut public_inputs = Vec::new();
        let delta_1 = U128x128::from(self.delta_1);
        dbg!(delta_1);
        //dbg!(delta_1.to_field_elements().unwrap());
        public_inputs.extend(delta_1.to_field_elements().unwrap());
        public_inputs.extend(U128x128::from(self.delta_2).to_field_elements().unwrap());
        public_inputs.extend(U128x128::from(self.lambda_1).to_field_elements().unwrap());
        public_inputs.extend(U128x128::from(self.lambda_2).to_field_elements().unwrap());
        public_inputs.extend(U128x128::from(self.unfilled_1).to_field_elements().unwrap());
        public_inputs.extend(U128x128::from(self.unfilled_2).to_field_elements().unwrap());
        public_inputs.extend(Fq::from(self.height).to_field_elements().unwrap());
        public_inputs.extend(self.trading_pair.to_field_elements().unwrap());
        public_inputs.extend(Fq::from(self.epoch_height).to_field_elements().unwrap());
        Some(public_inputs)
    }
}

pub struct BatchSwapOutputDataVar {
    pub delta_1: U128x128Var,
    pub delta_2: U128x128Var,
    pub lambda_1: U128x128Var,
    pub lambda_2: U128x128Var,
    pub unfilled_1: U128x128Var,
    pub unfilled_2: U128x128Var,
    pub height: FqVar,
    pub trading_pair: TradingPairVar,
    pub epoch_height: FqVar,
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
        dbg!(output_data.delta_1);
        let delta_1_fixpoint: U128x128 = output_data.delta_1.into();
        dbg!(delta_1_fixpoint);
        let delta_1 = U128x128Var::new_variable(cs.clone(), || Ok(delta_1_fixpoint), mode)?;
        use ark_r1cs_std::R1CSVar;
        dbg!(delta_1.value());
        dbg!(delta_1.limbs[0].value());
        dbg!(delta_1.limbs[1].value());
        dbg!(delta_1.limbs[2].value());
        dbg!(delta_1.limbs[3].value());

        let delta_2_fixpoint: U128x128 = output_data.delta_2.into();
        let delta_2 = U128x128Var::new_variable(cs.clone(), || Ok(delta_2_fixpoint), mode)?;
        let lambda_1_fixpoint: U128x128 = output_data.lambda_1.into();
        let lambda_1 = U128x128Var::new_variable(cs.clone(), || Ok(lambda_1_fixpoint), mode)?;
        let lambda_2_fixpoint: U128x128 = output_data.lambda_2.into();
        let lambda_2 = U128x128Var::new_variable(cs.clone(), || Ok(lambda_2_fixpoint), mode)?;
        let unfilled_1_fixpoint: U128x128 = output_data.unfilled_1.into();
        let unfilled_1 = U128x128Var::new_variable(cs.clone(), || Ok(unfilled_1_fixpoint), mode)?;
        let unfilled_2_fixpoint: U128x128 = output_data.unfilled_2.into();
        let unfilled_2 = U128x128Var::new_variable(cs.clone(), || Ok(unfilled_2_fixpoint), mode)?;
        let height = FqVar::new_variable(cs.clone(), || Ok(Fq::from(output_data.height)), mode)?;
        let trading_pair = TradingPairVar::new_variable_unchecked(
            cs.clone(),
            || Ok(output_data.trading_pair),
            mode,
        )?;
        let epoch_height =
            FqVar::new_variable(cs, || Ok(Fq::from(output_data.epoch_height)), mode)?;

        Ok(Self {
            delta_1,
            delta_2,
            lambda_1,
            lambda_2,
            unfilled_1,
            unfilled_2,
            trading_pair,
            height,
            epoch_height,
        })
    }
}

impl TypeUrl for BatchSwapOutputData {
    const TYPE_URL: &'static str = "/penumbra.dex.v1alpha1.BatchSwapOutputData";
}

impl DomainType for BatchSwapOutputData {
    type Proto = pb::BatchSwapOutputData;
}

impl From<BatchSwapOutputData> for pb::BatchSwapOutputData {
    fn from(s: BatchSwapOutputData) -> Self {
        pb::BatchSwapOutputData {
            delta_1: Some(s.delta_1.into()),
            delta_2: Some(s.delta_2.into()),
            lambda_1: Some(s.lambda_1.into()),
            lambda_2: Some(s.lambda_2.into()),
            unfilled_1: Some(s.unfilled_1.into()),
            unfilled_2: Some(s.unfilled_2.into()),
            height: s.height,
            epoch_height: s.epoch_height,
            trading_pair: Some(s.trading_pair.into()),
        }
    }
}

impl BatchSwapOutputDataVar {
    pub fn pro_rata_outputs(
        &self,
        delta_1_i: AmountVar,
        delta_2_i: AmountVar,
        cs: ConstraintSystemRef<Fq>,
    ) -> Result<(AmountVar, AmountVar), SynthesisError> {
        // The pro rata fraction is delta_j_i / delta_j, which we can multiply through:
        //   lambda_2_i = (delta_1_i / delta_1) * lambda_2   + (delta_2_i / delta_2) * unfilled_2
        //   lambda_1_i = (delta_1_i / delta_1) * unfilled_1 + (delta_2_i / delta_2) * lambda_1

        let delta_1_i = U128x128Var::from_amount_var(delta_1_i)?;
        let delta_2_i = U128x128Var::from_amount_var(delta_2_i)?;

        let zero = U128x128Var::zero();
        let one = U128x128Var::new_constant(cs.clone(), U128x128::from(1u64))?;

        // Compute the user i's share of the batch inputs of assets 1 and 2.
        // When the batch input delta_1 is zero, all pro-rata shares of it are also zero.
        let delta_1_is_zero = self.delta_1.is_eq(&zero)?;
        let divisor_1 = U128x128Var::conditionally_select(&delta_1_is_zero, &one, &self.delta_1)?;
        let division_result_1 = delta_1_i.checked_div(&divisor_1, cs.clone())?;
        let pro_rata_input_1 =
            U128x128Var::conditionally_select(&delta_1_is_zero, &zero, &division_result_1)?;

        let delta_2_is_zero = self.delta_2.is_eq(&zero)?;
        let divisor_2 = U128x128Var::conditionally_select(&delta_2_is_zero, &one, &self.delta_2)?;
        let division_result_2 = delta_2_i.checked_div(&divisor_2, cs)?;
        let pro_rata_input_2 =
            U128x128Var::conditionally_select(&delta_2_is_zero, &zero, &division_result_2)?;

        // let lambda_2_i = (pro_rata_input_1 * lambda_2).unwrap_or_default()
        //     + (pro_rata_input_2 * unfilled_2).unwrap_or_default();
        let addition_term2_1 = pro_rata_input_1.clone().checked_mul(&self.lambda_2)?;
        let addition_term2_2 = pro_rata_input_2.clone().checked_mul(&self.unfilled_2)?;
        let lambda_2_i = addition_term2_1.checked_add(&addition_term2_2)?;

        // let lambda_1_i = (pro_rata_input_1 * unfilled_1).unwrap_or_default()
        //     + (pro_rata_input_2 * lambda_1).unwrap_or_default();
        let addition_term1_1 = pro_rata_input_1.checked_mul(&self.unfilled_1)?;
        let addition_term1_2 = pro_rata_input_2.checked_mul(&self.lambda_1)?;
        let lambda_1_i = addition_term1_1.checked_add(&addition_term1_2)?;

        let lambda_1_i_rounded = lambda_1_i.round_down();
        let lambda_2_i_rounded = lambda_2_i.round_down();

        Ok((lambda_1_i_rounded.into(), lambda_2_i_rounded.into()))
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
            lambda_1: s
                .lambda_1
                .ok_or_else(|| anyhow!("Missing lambda_1"))?
                .try_into()?,
            lambda_2: s
                .lambda_2
                .ok_or_else(|| anyhow!("Missing lambda_2"))?
                .try_into()?,
            unfilled_1: s
                .unfilled_1
                .ok_or_else(|| anyhow!("Missing unfilled_1"))?
                .try_into()?,
            unfilled_2: s
                .unfilled_2
                .ok_or_else(|| anyhow!("Missing unfilled_2"))?
                .try_into()?,
            height: s.height,
            trading_pair: s
                .trading_pair
                .ok_or_else(|| anyhow!("Missing trading_pair"))?
                .try_into()?,
            epoch_height: s.epoch_height,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pasiphae_inflation_bug() {
        let bsod: BatchSwapOutputData = serde_json::from_str(
            r#"
{
    "delta1": {
        "lo": "31730032"
    },
    "delta2": {},
    "unfilled1": {},
    "lambda2": {
        "lo": "28766268"
    },
    "lambda1": {},
    "unfilled2": {},
    "height": "2185",
    "tradingPair": {
        "asset1": {
        "inner": "HW2Eq3UZVSBttoUwUi/MUtE7rr2UU7/UH500byp7OAc="
        },
        "asset2": {
        "inner": "KeqcLzNx9qSH5+lcJHBB9KNW+YPrBk5dKzvPMiypahA="
        }
    }
}"#,
        )
        .unwrap();

        let (delta_1_i, delta_2_i) = (Amount::from(31730032u64), Amount::from(0u64));

        let (lambda_1_i, lambda_2_i) = bsod.pro_rata_outputs((delta_1_i, delta_2_i));

        assert_eq!(lambda_1_i, Amount::from(0u64));
        assert_eq!(lambda_2_i, Amount::from(28766268u64));
    }
}
