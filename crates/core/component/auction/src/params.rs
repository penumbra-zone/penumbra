use penumbra_sdk_proto::core::component::auction::v1 as pb;
use penumbra_sdk_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::AuctionParameters", into = "pb::AuctionParameters")]
pub struct AuctionParameters {}

impl DomainType for AuctionParameters {
    type Proto = pb::AuctionParameters;
}

impl From<AuctionParameters> for pb::AuctionParameters {
    fn from(_: AuctionParameters) -> Self {
        pb::AuctionParameters {}
    }
}

impl TryFrom<pb::AuctionParameters> for AuctionParameters {
    type Error = anyhow::Error;

    fn try_from(_: pb::AuctionParameters) -> anyhow::Result<Self> {
        Ok(AuctionParameters {})
    }
}

impl Default for AuctionParameters {
    fn default() -> Self {
        AuctionParameters {}
    }
}
