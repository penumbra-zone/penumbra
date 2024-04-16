use anyhow::anyhow;
use penumbra_asset::{Balance, Value};
use penumbra_proto::{core::component::auction::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::auction::{id::AuctionId, AuctionNft};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ActionDutchAuctionEnd",
    into = "pb::ActionDutchAuctionEnd"
)]
pub struct ActionDutchAuctionEnd {
    pub auction_id: AuctionId,
}

impl ActionDutchAuctionEnd {
    pub fn balance(&self) -> Balance {
        let schedule_auction = Value {
            amount: 1u128.into(),
            asset_id: AuctionNft::new(self.auction_id, 0u64).asset_id(),
        };

        let end_auction = Value {
            amount: 1u128.into(),
            asset_id: AuctionNft::new(self.auction_id, 1u64).asset_id(),
        };

        Balance::from(end_auction) - Balance::from(schedule_auction)
    }
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
