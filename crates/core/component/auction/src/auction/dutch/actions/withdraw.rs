use crate::auction::dutch::DutchAuctionauction_id;
use anyhow::anyhow;
use penumbra_proto::{core::component::auction::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ActionDutchAuctionWithdraw",
    into = "pb::ActionDutchAuctionWithdraw"
)]
pub struct ActionDutchAuctionWithdraw {
    pub auction_id: AuctionId,
}

/* Protobuf impls */
impl DomainType for ActionDutchAuctionWithdraw {
    type Proto = pb::ActionDutchAuctionWithdraw;
}

impl From<ActionDutchAuctionWithdraw> for pb::ActionDutchAuctionWithdraw {
    fn from(domain: ActionDutchAuctionWithdraw) -> Self {
        pb::ActionDutchAuctionWithdraw {
            auction_id: Some(domain.auction_id.into()),
        }
    }
}

impl TryFrom<pb::ActionDutchAuctionWithdraw> for ActionDutchAuctionWithdraw {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ActionDutchAuctionWithdraw) -> Result<Self, Self::Error> {
        Ok(ActionDutchAuctionWithdraw {
            auction_id: msg
                .auction_id
                .ok_or_else(|| {
                    anyhow!("ActionDutchAuctionWithdraw message is missing an auction_id")
                })?
                .try_into()?,
        })
    }
}
