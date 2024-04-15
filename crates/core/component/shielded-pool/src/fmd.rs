use anyhow::{anyhow, Result};
use decaf377_fmd::Precision;
use penumbra_proto::{
    core::component::shielded_pool::v1::{self as pb},
    DomainType,
};
use serde::{Deserialize, Serialize};

pub mod state_key;

/// How long users have to switch to updated parameters.
pub const FMD_GRACE_PERIOD_BLOCKS_DEFAULT: u64 = 1 << 4;
/// How often we update the params, in terms of the number of grace periods
pub const FMD_UPDATE_FREQUENCY_GRACE_PERIOD: u64 = 4;

pub fn should_update_fmd_params(fmd_grace_period_blocks: u64, height: u64) -> bool {
    height % (fmd_grace_period_blocks * FMD_UPDATE_FREQUENCY_GRACE_PERIOD) == 0
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::FmdParameters", into = "pb::FmdParameters")]
pub struct Parameters {
    /// FMD Precision.
    pub precision: Precision,
    /// The block height at which these parameters became effective.
    pub as_of_block_height: u64,
}

impl DomainType for Parameters {
    type Proto = pb::FmdParameters;
}

impl TryFrom<pb::FmdParameters> for Parameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::FmdParameters) -> Result<Self> {
        Ok(Parameters {
            precision: msg.precision_bits.try_into()?,
            as_of_block_height: msg.as_of_block_height,
        })
    }
}

impl From<Parameters> for pb::FmdParameters {
    fn from(params: Parameters) -> Self {
        pb::FmdParameters {
            precision_bits: params.precision.bits() as u32,
            as_of_block_height: params.as_of_block_height,
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            precision: Precision::default(),
            as_of_block_height: 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaParametersAlgorithm {
    /// Used a fixed precision forever.
    Fixed(Precision),
}

/// Meta paramaters governing how FMD parameters change.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::FmdMetaParameters", into = "pb::FmdMetaParameters")]
pub struct MetaParameters {
    pub fmd_grace_period_blocks: u64,
    pub algorithm: MetaParametersAlgorithm,
}

impl TryFrom<pb::FmdMetaParameters> for MetaParameters {
    type Error = anyhow::Error;

    fn try_from(value: pb::FmdMetaParameters) -> Result<Self> {
        let fmd_grace_period_blocks = value.fmd_grace_period_blocks;
        let algorithm = match value
            .algorithm
            .ok_or(anyhow!("FmdMetaParameters missing algorithm"))?
        {
            pb::fmd_meta_parameters::Algorithm::FixedPrecisionBits(p) => {
                MetaParametersAlgorithm::Fixed(Precision::new(p as u8)?)
            }
        };
        Ok(MetaParameters {
            fmd_grace_period_blocks,
            algorithm,
        })
    }
}

impl From<MetaParameters> for pb::FmdMetaParameters {
    fn from(value: MetaParameters) -> Self {
        let algorithm = match value.algorithm {
            MetaParametersAlgorithm::Fixed(p) => {
                pb::fmd_meta_parameters::Algorithm::FixedPrecisionBits(p.bits().into())
            }
        };
        pb::FmdMetaParameters {
            fmd_grace_period_blocks: value.fmd_grace_period_blocks,
            algorithm: Some(algorithm),
        }
    }
}

impl DomainType for MetaParameters {
    type Proto = pb::FmdMetaParameters;
}

impl Default for MetaParameters {
    fn default() -> Self {
        Self {
            fmd_grace_period_blocks: FMD_GRACE_PERIOD_BLOCKS_DEFAULT,
            algorithm: MetaParametersAlgorithm::Fixed(Precision::default()),
        }
    }
}

impl MetaParameters {
    pub fn updated_fmd_params(
        &self,
        _old: &Parameters,
        height: u64,
        _clue_count_delta: (u64, u64),
    ) -> Parameters {
        match self.algorithm {
            MetaParametersAlgorithm::Fixed(precision) => Parameters {
                precision,
                as_of_block_height: height,
            },
        }
    }
}
