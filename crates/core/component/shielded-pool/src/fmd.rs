use penumbra_proto::{core::component::shielded_pool::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

pub mod state_key;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::FmdParameters", into = "pb::FmdParameters")]
pub struct Parameters {
    /// Bits of precision.
    pub precision_bits: u8,
    /// The block height at which these parameters became effective.
    pub as_of_block_height: u64,
}

impl DomainType for Parameters {
    type Proto = pb::FmdParameters;
}

impl TryFrom<pb::FmdParameters> for Parameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::FmdParameters) -> Result<Self, Self::Error> {
        Ok(Parameters {
            precision_bits: msg.precision_bits.try_into()?,
            as_of_block_height: msg.as_of_block_height,
        })
    }
}

impl From<Parameters> for pb::FmdParameters {
    fn from(params: Parameters) -> Self {
        pb::FmdParameters {
            precision_bits: u32::from(params.precision_bits),
            as_of_block_height: params.as_of_block_height,
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            precision_bits: 0,
            as_of_block_height: 1,
        }
    }
}
