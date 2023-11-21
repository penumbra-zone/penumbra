use penumbra_proto::penumbra::core::component::fee::v1alpha1 as pb;

use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(try_from = "pb::FeeParameters", into = "pb::FeeParameters")]
pub struct FeeParameters {}

impl DomainType for FeeParameters {
    type Proto = pb::FeeParameters;
}

impl TryFrom<pb::FeeParameters> for FeeParameters {
    type Error = anyhow::Error;

    fn try_from(_msg: pb::FeeParameters) -> anyhow::Result<Self> {
        Ok(FeeParameters {})
    }
}

impl From<FeeParameters> for pb::FeeParameters {
    fn from(_params: FeeParameters) -> Self {
        pb::FeeParameters {}
    }
}
