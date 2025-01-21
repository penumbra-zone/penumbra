use penumbra_sdk_proto::penumbra::core::component::shielded_pool::v1 as pb;

use penumbra_sdk_proto::DomainType;
use serde::{Deserialize, Serialize};

use crate::fmd;

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(
    try_from = "pb::ShieldedPoolParameters",
    into = "pb::ShieldedPoolParameters"
)]
pub struct ShieldedPoolParameters {
    pub fmd_meta_params: fmd::MetaParameters,
}

impl DomainType for ShieldedPoolParameters {
    type Proto = pb::ShieldedPoolParameters;
}

impl TryFrom<pb::ShieldedPoolParameters> for ShieldedPoolParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ShieldedPoolParameters) -> anyhow::Result<Self> {
        Ok(ShieldedPoolParameters {
            fmd_meta_params: msg
                .fmd_meta_params
                .ok_or_else(|| anyhow::anyhow!("missing fmd_meta_params"))?
                .try_into()?,
        })
    }
}

impl From<ShieldedPoolParameters> for pb::ShieldedPoolParameters {
    fn from(params: ShieldedPoolParameters) -> Self {
        #[allow(deprecated)]
        pb::ShieldedPoolParameters {
            fmd_meta_params: Some(params.fmd_meta_params.into()),
            fixed_fmd_params: None,
        }
    }
}
