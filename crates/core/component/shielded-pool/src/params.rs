use penumbra_proto::penumbra::core::component::fee::v1alpha1 as pb;

use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(
    try_from = "pb::ShieldedPoolParameters",
    into = "pb::ShieldedPoolParameters"
)]
pub struct ShieldedPoolParameters {
    pub fmd_parameters: fmd::Parameters,
}

impl DomainType for ShieldedPoolParameters {
    type Proto = pb::ShieldedPoolParameters;
}

impl TryFrom<pb::ShieldedPoolParameters> for ShieldedPoolParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ShieldedPoolParameters) -> anyhow::Result<Self> {
        Ok(ShieldedPoolParameters {
            fmd_parameters: msg
                .fmd_parameters
                .ok_or_else(|| anyhow::anyhow!("missing fmd_parameters"))?
                .try_into()?,
            s,
        })
    }
}

impl From<ShieldedPoolParameters> for pb::ShieldedPoolParameters {
    fn from(params: ShieldedPoolParameters) -> Self {
        pb::ShieldedPoolParameters {
            fmd_parameters: Some(params.fmd_parameters.into()),
        }
    }
}
