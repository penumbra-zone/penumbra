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

        })
    }
}

impl From<ShieldedPoolParameters> for pb::ShieldedPoolParameters {
    fn from(_params: ShieldedPoolParameters) -> Self {
        pb::ShieldedPoolParameters {}
    }
}
