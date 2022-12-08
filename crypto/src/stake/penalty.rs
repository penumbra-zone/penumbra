use std::str::FromStr;

use penumbra_proto::{core::stake::v1alpha1 as pbs, Protobuf};
use serde::{Deserialize, Serialize};

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
