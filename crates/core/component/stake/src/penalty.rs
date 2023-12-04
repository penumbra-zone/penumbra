use std::str::FromStr;

use ark_ff::ToConstraintField;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::{r1cs::FqVar, FieldExt, Fq};
use penumbra_proto::{penumbra::core::component::stake::v1alpha1 as pbs, DomainType};
use serde::{Deserialize, Serialize};

use penumbra_asset::{
    asset::{self, AssetIdVar},
    balance::BalanceVar,
    Balance, Value, ValueVar, STAKING_TOKEN_ASSET_ID,
};
use penumbra_num::fixpoint::U128x128Var;
use penumbra_num::{
    fixpoint::{bit_constrain, U128x128},
    Amount, AmountVar,
};

/// Tracks slashing penalties applied to a validator in some epoch.
///
/// Note(erwan): we are trying to get a sense of what a `U128x128` based penalty would
/// look like. However, since this might be a bit too much for the undelegate claim
/// circuit to handle, we can keep using an `Amount` in our back pocket.
///
/// Another thing to consider is to either limit the magnitude of the penalty, or change the
/// API to propagate errors. Many operations are simply infallible if we assume that the penalty
/// is at most 100%. It also aligns with how penalties are used in practice. TODO(erwan).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(try_from = "pbs::Penalty", into = "pbs::Penalty")]
pub struct Penalty(pub U128x128);

impl Penalty {
    pub fn from_percent(percent: u64) -> Self {
        Penalty(U128x128::ratio(percent as u128, 100u128).expect("infaillible"))
    }
    pub fn from_bps(bps: u64) -> Self {
        Penalty(U128x128::ratio(bps as u128, 10000u128).expect("infaillible"))
    }

    /// Compound this `Penalty` with another `Penalty`.
    pub fn compound(&self, other: Penalty) -> Penalty {
        /* Compounding:
         * For two penalty rates p1 and p2, the compounded penalty rate q is:
         *     `1 - q = (1-p1)(1-p2)``
         *     `q = 1 - (1-p1)(1-p2)`
         */
        let one = U128x128::from(1u128);
        let p1 = self.0;
        let p2 = other.0;
        let p1_retention = (one - p1).expect("p1 is less than 1");
        let p2_retention = (one - p2).expect("p2 is less than 1");
        let q = one - (p1_retention * p2_retention).expect("cannot overflow");
        let q = q.expect("product of two factors less than 1 is less than 1");
        Penalty(q)
    }

    /// Apply this `Penalty` to an `Amount` of unbonding tokens.
    pub fn apply_to(&self, amount: Amount) -> Amount {
        let amount = U128x128::from(amount);
        let penalized_amount = (amount * self.0).expect("cannot overflow");
        penalized_amount
            .round_down()
            .try_into()
            .expect("integral value")
    }

    /// Helper method to compute the effect of an UndelegateClaim on the
    /// transaction's value balance, used in planning and (transparent) proof
    /// verification.
    ///
    /// This method takes the `unbonding_id` rather than the `UnbondingToken` so
    /// that it can be used in mock proof verification, where computation of the
    /// unbonding token's asset ID happens outside of the circuit.
    pub fn balance_for_claim(&self, unbonding_id: asset::Id, unbonding_amount: Amount) -> Balance {
        // The undelegate claim action subtracts the unbonding amount and adds
        // the unbonded amount from the transaction's value balance.
        Balance::zero()
            - Value {
                amount: unbonding_amount,
                asset_id: unbonding_id,
            }
            + Value {
                amount: self.apply_to(unbonding_amount),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            }
    }
}

impl ToConstraintField<Fq> for Penalty {
    fn to_field_elements(&self) -> Option<Vec<Fq>> {
        let field_elements = self.0.to_field_elements()?;
        Some(field_elements)
    }
}

pub struct PenaltyVar {
    inner: U128x128Var,
}

impl AllocVar<Penalty, Fq> for PenaltyVar {
    fn new_variable<T: std::borrow::Borrow<Penalty>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner: Penalty = *f()?.borrow();
        let penalty = U128x128Var::new_variable(cs, || Ok(inner.0), mode)?;

        Ok(Self { inner: penalty })
    }
}

impl PenaltyVar {
    pub fn apply_to(&self, staked_amount: AmountVar) -> Result<AmountVar, SynthesisError> {
        /* Bound analysis
         *      `penalty_amount = amount * (1_0000_0000 - penalty) / 1_0000_0000`
         * Order of operations:
         *      1. cst:  `penalty` cast to u128 (infallible)
         *      2. sub:  `1_0000_0000 - penalty`
         *      3. mul:  `amount * (1_0000_0000 - penalty)`
         *      4. div:  `amount * (1_0000_0000 - penalty) / 1_0000_0000`
         *
         * Units:
         *      `amount`                    : staking tokens to undelegate (128 bits)
         *      `penalty`                   : a bps^2 penalty factor between 0 and 10^8 (64 bits)
         *      `staking_token_unit_amount` : 10^6 ~ 2^20
         *      `bps_squared_constant`      : 10^8 ~ 2^27
         *
         * Overflow condition: `amount * (1_0000_0000 - penalty) > 2^128 - 1`
         * Undeflow condition: `penalty` > 10^8 (penalty is greater than 100%)
         *
         * Boundary: If penalty is 0, then `amount * 1_0000_0000 = amount * 2^27`
         * With `amount` as 2^(x+20) - 1, where x is log2(staking tokens):
         *    What quantity of staking tokens would cause an overflow? (for 128 bits)
         *    Find x: 2^(x+20) * 2^27 < 2^128
         *    True for x < 81 (~10^24 staking tokens), so an overflow for 128 bits is implausible.
         *
         */

        // This is the same as the `Penalty::apply_to` impl but in circuit
        let staked_amount = U128x128Var::from_amount_var(staked_amount)?;
        let penalized_amount = staked_amount.checked_mul(&self.inner)?;

        // Round down and construct a AmountVar
        let rounded_penalized_amount = penalized_amount.round_down_to_amount()?;

        Ok(rounded_penalized_amount)
    }

    pub fn balance_for_claim(
        &self,
        unbonding_id: AssetIdVar,
        unbonding_amount: AmountVar,
    ) -> Result<BalanceVar, SynthesisError> {
        let negative_value = BalanceVar::from_negative_value_var(ValueVar {
            amount: unbonding_amount.clone(),
            asset_id: unbonding_id,
        });
        let staking_token_asset_id_var =
            AssetIdVar::new_witness(self.cs(), || Ok(*STAKING_TOKEN_ASSET_ID))?;

        let positive_value = BalanceVar::from_positive_value_var(ValueVar {
            amount: self.apply_to(unbonding_amount)?,
            asset_id: staking_token_asset_id_var,
        });
        Ok(negative_value + positive_value)
    }
}

impl R1CSVar<Fq> for PenaltyVar {
    type Value = Penalty;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.inner.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        Ok(Penalty(self.inner.value()?))
    }
}

impl FromStr for Penalty {
    type Err = <u64 as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = u64::from_str(s)?;
        //Ok(Penalty(v))
        todo!()
    }
}

impl std::fmt::Display for Penalty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainType for Penalty {
    type Proto = pbs::Penalty;
}

impl From<Penalty> for pbs::Penalty {
    fn from(v: Penalty) -> Self {
        todo!()
        //pbs::Penalty { inner: v.0 }
    }
}

impl TryFrom<pbs::Penalty> for Penalty {
    type Error = anyhow::Error;
    fn try_from(v: pbs::Penalty) -> Result<Self, Self::Error> {
        todo!()
        // Ok(Penalty(v.inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn penalty_display_fromstr_roundtrip() {
        let p = Penalty(123456789u128.into());
        let s = p.to_string();
        let p2 = Penalty::from_str(&s).unwrap();
        assert_eq!(p, p2);
    }
}
