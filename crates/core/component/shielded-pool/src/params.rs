use penumbra_proto::penumbra::core::component::shielded_pool::v1 as pb;

use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

use crate::fmd;

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(
    try_from = "pb::ShieldedPoolParameters",
    into = "pb::ShieldedPoolParameters"
)]
pub struct ShieldedPoolParameters {
    pub fixed_fmd_params: fmd::Parameters,
}

impl DomainType for ShieldedPoolParameters {
    type Proto = pb::ShieldedPoolParameters;
}

impl TryFrom<pb::ShieldedPoolParameters> for ShieldedPoolParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ShieldedPoolParameters) -> anyhow::Result<Self> {
        Ok(ShieldedPoolParameters {
            fixed_fmd_params: msg
                .fixed_fmd_params
                .ok_or_else(|| anyhow::anyhow!("missing fmd_parameters"))?
                .try_into()?,
        })
    }
}

impl From<ShieldedPoolParameters> for pb::ShieldedPoolParameters {
    fn from(params: ShieldedPoolParameters) -> Self {
        pb::ShieldedPoolParameters {
            fixed_fmd_params: Some(params.fixed_fmd_params.into()),
        }
    }
}
