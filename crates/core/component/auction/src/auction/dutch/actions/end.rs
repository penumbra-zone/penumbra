use crate::auction::dutch::DutchAuctionauction_id;
use anyhow::anyhow;
use penumbra_proto::{core::component::auction::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ActionDutchAuctionEnd",
    into = "pb::ActionDutchAuctionEnd"
)]
pub struct ActionDutchAuctionEnd {
    pub auction_id: AuctionId,
}

/* Protobuf impls */
impl DomainType for ActionDutchAuctionEnd {
    type Proto = pb::ActionDutchAuctionEnd;
}

impl From<ActionDutchAuctionEnd> for pb::ActionDutchAuctionEnd {
    fn from(domain: ActionDutchAuctionEnd) -> Self {
        pb::ActionDutchAuctionEnd {
            auction_id: Some(domain.auction_id.into()),
        }
    }
}

impl TryFrom<pb::ActionDutchAuctionEnd> for ActionDutchAuctionEnd {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ActionDutchAuctionEnd) -> Result<Self, Self::Error> {
        Ok(ActionDutchAuctionEnd {
            auction_id: msg
                .auction_id
                .ok_or_else(|| anyhow!("ActionDutchAuctionEnd message is missing an auction_id"))?
                .try_into()?,
        })
    }
}
