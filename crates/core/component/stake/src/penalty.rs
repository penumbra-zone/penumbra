use ark_ff::ToConstraintField;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::Fq;
use penumbra_proto::{penumbra::core::component::stake::v1alpha1 as pbs, DomainType};
use serde::{Deserialize, Serialize};

use penumbra_asset::{
    asset::{self, AssetIdVar},
    balance::BalanceVar,
    Balance, Value, ValueVar, STAKING_TOKEN_ASSET_ID,
};
use penumbra_num::{
    fixpoint::{U128x128, U128x128Var},
    Amount, AmountVar,
};

/// Tracks slashing penalties applied to a validator in some epoch.
///
/// The penalty is represented as a fixed-point integer in bps^2 (denominator 10^8).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(try_from = "pbs::Penalty", into = "pbs::Penalty")]
pub struct Penalty(U128x128);

impl Penalty {
    /// Create a `Penalty` from a percentage e.g.
    /// `Penalty::from_percent(1)` is a 1% penalty.
    /// `Penalty::from_percent(100)` is a 100% penalty.
    pub fn from_percent(percent: u64) -> Self {
        Penalty::from_bps(percent.saturating_mul(100))
    }

    /// Create a `Penalty` from a basis point e.g.
    /// `Penalty::from_bps(1)` is a 1 bps penalty.
    /// `Penalty::from_bps(100)` is a 100 bps penalty.
    pub fn from_bps(bps: u64) -> Self {
        Penalty::from_bps_squared(bps.saturating_mul(10000))
    }

    /// Create a `Penalty` from a basis point squared e.g.
    /// `Penalty::from_bps(1_0000_0000)` is a 100% penalty.
    pub fn from_bps_squared(bps_squared: u64) -> Self {
        assert!(bps_squared <= 1_0000_0000);
        Self(U128x128::ratio(bps_squared, 1_0000_0000).expect(&format!(
            "{bps_squared} bps^2 should be convertible to a U128x128"
        )))
    }

    fn one_minus_this(&self) -> Penalty {
        Self(
            (U128x128::from(1u64) - self.0)
                .expect("1 - penalty should never underflow, because penalty is at most 1"),
        )
    }

    /// Compound this `Penalty` with another `Penalty`.
    pub fn compound(&self, other: Penalty) -> Penalty {
        // We want to compute q sth (1 - q) = (1-p1)(1-p2)
        // q = 1 - (1-p1)(1-p2)
        Self(
            (self.one_minus_this().0 * other.one_minus_this().0)
                .expect("compounding penalties will never overflow, both are <= 1"),
        )
        .one_minus_this()
    }

    /// Apply this `Penalty` to an `Amount` of unbonding tokens.
    pub fn apply_to(&self, amount: Amount) -> Amount {
        (U128x128::from(amount) * self.one_minus_this().0)
            .expect("should not overflow, because penalty is <= 1")
            .round_down()
            .try_into()
            .expect("converting integral U128xU128 into Amount will succeed")
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
        self.0.to_field_elements()
    }
}

impl From<Penalty> for U128x128 {
    fn from(value: Penalty) -> Self {
        value.0
    }
}

impl From<Penalty> for [u8; 32] {
    fn from(value: Penalty) -> Self {
        value.0.into()
    }
}

impl<'a> TryFrom<&'a [u8]> for Penalty {
    type Error = <U128x128 as TryFrom<&'a [u8]>>::Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        U128x128::try_from(value).map(Self)
    }
}

/// One explicit choice in this in circuit representation is that we
/// DO NOT CHECK THAT THE PENALTY IS <= 1 IN CIRCUIT. This is in practice
/// the UndelegateClaim circuit is the ONLY consumer of this type, and
/// in the context of that circuit, the penalty is checked out of circuit
/// to conform to a real value which will be <= 1.
///
/// I repeat myself: IF YOU EVER USE THIS IN A DIFFERENT CIRCUIT, MAKE ABSOLUTELY
/// CERTAIN THAT A PENALTY BEING > 1 IN CIRCUIT IS NOT AN ISSUE.
pub struct PenaltyVar {
    inner: U128x128Var,
}

impl AllocVar<Penalty, Fq> for PenaltyVar {
    fn new_variable<T: std::borrow::Borrow<Penalty>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        Ok(Self {
            inner: U128x128Var::new_variable(cs, || Ok(f()?.borrow().0), mode)?,
        })
    }
}

impl PenaltyVar {
    fn one_minus_this(&self) -> Result<PenaltyVar, SynthesisError> {
        // Calculate 1 - self outside the circuit
        let ooc_result = {
            let value = self.value().unwrap_or(Penalty::default());
            PenaltyVar::new_witness(self.cs(), || Ok(value.one_minus_this()))?
        };
        // Check that 1 + (1 - self) is self
        let one = PenaltyVar::new_constant(self.cs(), Penalty::from_percent(100))?;
        self.inner
            .enforce_equal(&one.inner.checked_add(&ooc_result.inner)?)?;
        Ok(ooc_result)
    }

    pub fn apply_to(&self, amount: AmountVar) -> Result<AmountVar, SynthesisError> {
        U128x128Var::from_amount_var(amount)?
            .checked_mul(&self.one_minus_this()?.inner)?
            .round_down_to_amount()
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

impl DomainType for Penalty {
    type Proto = pbs::Penalty;
}

impl From<Penalty> for pbs::Penalty {
    fn from(v: Penalty) -> Self {
        pbs::Penalty {
            inner: <[u8; 32]>::from(v).to_vec(),
        }
    }
}

impl TryFrom<pbs::Penalty> for Penalty {
    type Error = anyhow::Error;
    fn try_from(v: pbs::Penalty) -> Result<Self, Self::Error> {
        Ok(Penalty::try_from(v.inner.as_slice())?)
    }
}
