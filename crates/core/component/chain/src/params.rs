use core::fmt;
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    str::FromStr,
};

// TODO(proto): eliminate these imports
use penumbra_proto::penumbra::core::component::chain::v1alpha1 as pb_chain;

use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    try_from = "pb_chain::ChainParameters",
    into = "pb_chain::ChainParameters"
)]
pub struct ChainParameters {
    pub chain_id: String,
    pub epoch_duration: u64,
}

impl DomainType for ChainParameters {
    type Proto = pb_chain::ChainParameters;
}

impl TryFrom<pb_chain::ChainParameters> for ChainParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb_chain::ChainParameters) -> anyhow::Result<Self> {
        Ok(ChainParameters {
            chain_id: msg.chain_id,
            epoch_duration: msg.epoch_duration,
        })
    }
}

impl From<ChainParameters> for pb_chain::ChainParameters {
    fn from(params: ChainParameters) -> Self {
        pb_chain::ChainParameters {
            chain_id: params.chain_id,
            epoch_duration: params.epoch_duration,
        }
    }
}

// TODO: defaults are implemented here as well as in the
// `pd::main`
impl Default for ChainParameters {
    fn default() -> Self {
        Self {
            chain_id: String::new(),
            epoch_duration: 719,
        }
    }
}

/// This is a ratio of two `u64` values, intended to be used solely in governance parameters and
/// tallying. It only implements construction and comparison, not arithmetic, to reduce the trusted
/// codebase for governance.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb_chain::Ratio", into = "pb_chain::Ratio")]
pub struct Ratio {
    numerator: u64,
    denominator: u64,
}

impl Display for Ratio {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

impl FromStr for Ratio {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('/');
        let numerator = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing numerator"))?
            .parse()?;
        let denominator = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing denominator"))?
            .parse()?;
        if parts.next().is_some() {
            anyhow::bail!("too many parts");
        }
        Ok(Ratio {
            numerator,
            denominator,
        })
    }
}

impl Ratio {
    pub fn new(numerator: u64, denominator: u64) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl PartialEq for Ratio {
    fn eq(&self, other: &Self) -> bool {
        // Convert everything to `u128` to avoid overflow when multiplying
        u128::from(self.numerator) * u128::from(other.denominator)
            == u128::from(self.denominator) * u128::from(other.numerator)
    }
}

impl Eq for Ratio {}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ratio {
    fn cmp(&self, other: &Self) -> Ordering {
        // Convert everything to `u128` to avoid overflow when multiplying
        (u128::from(self.numerator) * u128::from(other.denominator))
            .cmp(&(u128::from(self.denominator) * u128::from(other.numerator)))
    }
}

impl From<Ratio> for pb_chain::Ratio {
    fn from(ratio: Ratio) -> Self {
        pb_chain::Ratio {
            numerator: ratio.numerator,
            denominator: ratio.denominator,
        }
    }
}

impl From<pb_chain::Ratio> for Ratio {
    fn from(msg: pb_chain::Ratio) -> Self {
        Ratio {
            numerator: msg.numerator,
            denominator: msg.denominator,
        }
    }
}
