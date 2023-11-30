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
use penumbra_num::{fixpoint::bit_constrain, Amount, AmountVar};

/// Tracks slashing penalties applied to a validator in some epoch.
///
/// The penalty is represented as a fixed-point integer in bps^2 (denominator 10^8).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(try_from = "pbs::Penalty", into = "pbs::Penalty")]
pub struct Penalty(pub u64);

impl Penalty {
    /// Create a `Penalty` from a percentage e.g.
    /// `Penalty::from_percent(1)` is a 1% penalty.
    /// `Penalty::from_percent(100)` is a 100% penalty.
    pub fn from_percent(percent: u64) -> Self {
        Penalty::from_bps(percent * 100)
    }

    /// Create a `Penalty` from a basis point e.g.
    /// `Penalty::from_bps(1)` is a 1 bps penalty.
    /// `Penalty::from_bps(100)` is a 100 bps penalty.
    pub fn from_bps(bps: u64) -> Self {
        Penalty(bps * 10_000)
    }

    /// Compound this `Penalty` with another `Penalty`.
    pub fn compound(&self, other: Penalty) -> Penalty {
        // We want to compute q sth (1 - q) = (1-p1)(1-p2)
        // q = 1 - (1-p1)(1-p2)
        // but since each p_i implicitly carries a factor of 10^8, we need to divide by 10^8 after multiplying.
        let one = 1_0000_0000u128;
        let p1 = self.0 as u128;
        let p2 = other.0 as u128;
        let q = u64::try_from(one - (((one - p1) * (one - p2)) / 1_0000_0000))
            .expect("value should fit in 64 bits");
        Penalty(q)
    }

    /// Apply this `Penalty` to an `Amount` of unbonding tokens.
    pub fn apply_to(&self, amount: Amount) -> Amount {
        let penalized_amount = (u128::try_from(amount).expect("amount should be a valid u128"))
            * (1_0000_0000 - self.0 as u128)
            / 1_0000_0000;
        Amount::try_from(
            u128::try_from(penalized_amount).expect("penalized amount should fit in u128"),
        )
        .expect("all u128 values should be valid Amounts")
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
        let field_elements = vec![Fq::from(self.0)];
        Some(field_elements)
    }
}

pub struct PenaltyVar {
    inner: FqVar,
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
        let penalty = FqVar::new_variable(cs, || Ok(Fq::from(inner.0)), mode)?;
        // Check the Penalty is 64 bits
        let _ = bit_constrain(penalty.clone(), 64);
        Ok(Self { inner: penalty })
    }
}

impl From<&PenaltyVar> for AmountVar {
    fn from(penalty_var: &PenaltyVar) -> Self {
        // `AmountVar`s must fit in 128 bits, but `PenaltyVar`s have already
        // been constrained to fit in 64 bits, so we can safely
        // construct an `AmountVar` from a `PenaltyVar`.
        AmountVar {
            amount: penalty_var.inner.clone(),
        }
    }
}

impl PenaltyVar {
    pub fn apply_to(&self, amount: AmountVar) -> Result<AmountVar, SynthesisError> {
        let penalty = self.value().unwrap_or(Penalty(0));
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
         *    What quantity of staking tokens would cause an overflow? (for 64 bits)
         *    Find x: 2^(x+20) * 2^27 < 2^64
         *    True for x < 17 (~10^5 staking tokens), so an overflow for 64 bits is possible and plausible.
         *
         */

        // Out of circuit penalized amount computation:
        let amount_bytes = &amount.value().unwrap_or(Amount::from(0u64)).to_le_bytes()[0..16];
        let amount_128 =
            u128::from_le_bytes(amount_bytes.try_into().expect("should fit in 16 bytes"));
        let penalized_amount = amount_128 * (1_0000_0000 - penalty.0 as u128) / 1_0000_0000;

        // Witness the result in the circuit.
        let penalized_amount_var = AmountVar::new_witness(self.cs(), || {
            Ok(Amount::from(
                u128::try_from(penalized_amount).expect("can fit in u64"),
            ))
        })?;

        // Now we certify the witnessed penalized amount was calculated correctly.
        // Constrain: penalized_amount = amount * (1_0000_0000 - penalty (public)) / 1_0000_0000
        let hundred_mil = AmountVar::new_constant(self.cs(), Amount::from(1_0000_0000u128))?; // 1_0000_0000
        let numerator = amount * (hundred_mil.clone() - self.into());
        let (penalized_amount_quo, _) = numerator.quo_rem(&hundred_mil)?;
        penalized_amount_quo.enforce_equal(&penalized_amount_var)?;

        Ok(penalized_amount_var)
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
        let inner_fq = self.inner.value()?;
        let inner_bytes = &inner_fq.to_bytes()[0..8];
        let penalty_bytes: [u8; 8] = inner_bytes
            .try_into()
            .expect("should be able to fit in 16 bytes");
        Ok(Penalty(u64::from_le_bytes(penalty_bytes)))
    }
}

impl FromStr for Penalty {
    type Err = <u64 as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = u64::from_str(s)?;
        Ok(Penalty(v))
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
        pbs::Penalty { inner: v.0 }
    }
}

impl TryFrom<pbs::Penalty> for Penalty {
    type Error = anyhow::Error;
    fn try_from(v: pbs::Penalty) -> Result<Self, Self::Error> {
        Ok(Penalty(v.inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn penalty_display_fromstr_roundtrip() {
        let p = Penalty(123456789);
        let s = p.to_string();
        let p2 = Penalty::from_str(&s).unwrap();
        assert_eq!(p, p2);
    }
}
