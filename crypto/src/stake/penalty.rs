use std::str::FromStr;

use penumbra_proto::{core::stake::v1alpha1 as pbs, Protobuf};
use serde::{Deserialize, Serialize};

use crate::{asset, Amount, Balance, Value, STAKING_TOKEN_ASSET_ID};

/// Tracks slashing penalties applied to a validator in some epoch.
///
/// The penalty is represented as a fixed-point integer in bps^2 (denominator 10^8).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pbs::Penalty", into = "pbs::Penalty")]
pub struct Penalty(pub u64);

impl Default for Penalty {
    fn default() -> Self {
        Penalty(0)
    }
}

impl Penalty {
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
        // TODO: need widening Amount mul to handle 128-bit values, but we don't do that for staking anyways
        // TODO: this should all be infallible
        let penalized_amount =
            (u128::try_from(amount).unwrap()) * (1_0000_0000 - self.0 as u128) / 1_0000_0000;
        Amount::try_from(u64::try_from(penalized_amount).unwrap()).unwrap()
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
        let balance = Balance::zero()
            - Value {
                amount: unbonding_amount,
                asset_id: unbonding_id,
            }
            + Value {
                amount: self.apply_to(unbonding_amount),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            };

        balance
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

impl Protobuf<pbs::Penalty> for Penalty {}

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
