use crate::auction::{id::AuctionId, AuctionNft};
use anyhow::anyhow;
use penumbra_asset::{balance, Balance, Value};
use penumbra_proto::{core::component::auction::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ActionDutchAuctionWithdraw",
    into = "pb::ActionDutchAuctionWithdraw"
)]
pub struct ActionDutchAuctionWithdraw {
    pub auction_id: AuctionId,
    pub seq: u64,
    pub reserves_commitment: balance::Commitment,
}

impl ActionDutchAuctionWithdraw {
    pub fn balance(&self) -> Balance {
        let prev_auction_nft: Balance = Value {
            amount: 1u128.into(),
            asset_id: AuctionNft::new(self.auction_id, self.seq.saturating_sub(1)).asset_id(),
        }
        .into();

        let next_auction_nft: Balance = Value {
            amount: 1u128.into(),
            asset_id: AuctionNft::new(self.auction_id, self.seq).asset_id(),
        }
        .into();

        next_auction_nft - prev_auction_nft
    }
}

/* Protobuf impls */
impl DomainType for ActionDutchAuctionWithdraw {
    type Proto = pb::ActionDutchAuctionWithdraw;
}

impl From<ActionDutchAuctionWithdraw> for pb::ActionDutchAuctionWithdraw {
    fn from(domain: ActionDutchAuctionWithdraw) -> Self {
        pb::ActionDutchAuctionWithdraw {
            auction_id: Some(domain.auction_id.into()),
            seq: domain.seq,
            reserves_commitment: Some(domain.reserves_commitment.into()),
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
            seq: msg.seq,
            reserves_commitment: msg
                .reserves_commitment
                .ok_or_else(|| {
                    anyhow!("ActionDutchAuctionWithdraw message is missing reserves_commitment")
                })?
                .try_into()?,
        })
    }
}
