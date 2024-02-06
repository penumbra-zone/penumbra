use penumbra_proto::core::component::funding::v1 as pb;
use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::FundingParameters", into = "pb::FundingParameters")]
pub struct FundingParameters {}

impl DomainType for FundingParameters {
    type Proto = pb::FundingParameters;
}

impl TryFrom<pb::FundingParameters> for FundingParameters {
    type Error = anyhow::Error;

    fn try_from(_params: pb::FundingParameters) -> anyhow::Result<Self> {
        Ok(FundingParameters {})
    }
}

impl From<FundingParameters> for pb::FundingParameters {
    fn from(_params: FundingParameters) -> Self {
        pb::FundingParameters {}
    }
}

impl Default for FundingParameters {
    fn default() -> Self {
        Self {}
    }
}
