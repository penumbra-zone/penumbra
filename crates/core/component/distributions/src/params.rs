use penumbra_proto::core::component::distributions::v1alpha1 as pb;
use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    try_from = "pb::DistributionsParameters",
    into = "pb::DistributionsParameters"
)]
pub struct DistributionsParameters {
    pub staking_issuance_per_block: u64,
}

impl DomainType for DistributionsParameters {
    type Proto = pb::DistributionsParameters;
}

impl TryFrom<pb::DistributionsParameters> for DistributionsParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::DistributionsParameters) -> anyhow::Result<Self> {
        Ok(DistributionsParameters {
            staking_issuance_per_block: msg.staking_issuance_per_block,
        })
    }
}

impl From<DistributionsParameters> for pb::DistributionsParameters {
    fn from(params: DistributionsParameters) -> Self {
        pb::DistributionsParameters {
            staking_issuance_per_block: params.staking_issuance_per_block,
        }
    }
}

impl Default for DistributionsParameters {
    fn default() -> Self {
        Self {
            staking_issuance_per_block: 1,
        }
    }
}
