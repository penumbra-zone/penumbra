// use penumbra_proto::core::component::auction::v1 as pb;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
// #[serde(try_from = "pb::AuctionParameters", into = "pb::AuctionParameters")]
pub struct AuctionParameters {}

// impl DomainType for AuctionParameters {
//     type Proto = pb::AuctionParameters;
// }

// impl TryFrom<pb::AuctionParameters> for AuctionParameters {
//     type Error = anyhow::Error;
//
//     fn try_from(msg: pb::AuctionParameters) -> anyhow::Result<Self> {
//         Ok(AuctionParameters {})
//     }
// }
