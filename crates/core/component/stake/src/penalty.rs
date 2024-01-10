use ark_ff::ToConstraintField;
use decaf377::Fq;
use penumbra_proto::{penumbra::core::component::stake::v1alpha1 as pbs, DomainType};
use serde::{Deserialize, Serialize};

use penumbra_asset::{asset, Balance, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_num::{fixpoint::U128x128, Amount};

/// Tracks slashing penalties applied to a validator in some epoch.
///
/// You do not need to know how the penalty is represented.
///
/// If you insist on knowing, it's represented as a U128x128 between 0 and 1,
/// which denotes the amount *kept* after applying a penalty. e.g. a 1% penalty
/// would be 0.99.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
        .one_minus_this()
    }

    fn one_minus_this(&self) -> Penalty {
        Self(
            (U128x128::from(1u64) - self.0)
                .expect("1 - penalty should never underflow, because penalty is at most 1"),
        )
    }

    /// A rate representing how much of an asset remains after applying a penalty.
    ///
    /// e.g. a 1% penalty will yield a rate of 0.99 here.
    pub fn kept_rate(&self) -> U128x128 {
        self.0
    }

    /// Compound this `Penalty` with another `Penalty`.
    pub fn compound(&self, other: Penalty) -> Penalty {
        Self((self.0 * other.0).expect("compounding penalities will not overflow"))
    }

    /// Apply this `Penalty` to an `Amount` of unbonding tokens.
    pub fn apply_to_amount(&self, amount: Amount) -> Amount {
        self.0
            .apply_to_amount(&amount)
            .expect("should not overflow, because penalty is <= 1")
    }

    /// Apply this `Penalty` to some fraction.
    pub fn apply_to(&self, amount: impl Into<U128x128>) -> U128x128 {
        (amount.into() * self.0).expect("should not overflow, because penalty is <= 1")
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
                amount: self.apply_to_amount(unbonding_amount),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            }
    }
}

impl ToConstraintField<Fq> for Penalty {
    fn to_field_elements(&self) -> Option<Vec<Fq>> {
        self.0.to_field_elements()
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
