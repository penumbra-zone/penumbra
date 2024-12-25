use ark_ff::Zero;
use decaf377::Fr;
use penumbra_sdk_asset::{balance, Balance, Value};
use penumbra_sdk_proto::{penumbra::core::component::auction::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::auction::{dutch::ActionDutchAuctionWithdraw, AuctionId, AuctionNft};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    try_from = "pb::ActionDutchAuctionWithdrawPlan",
    into = "pb::ActionDutchAuctionWithdrawPlan"
)]
pub struct ActionDutchAuctionWithdrawPlan {
    pub auction_id: AuctionId,
    pub seq: u64,
    pub reserves_input: Value,
    pub reserves_output: Value,
}

impl ActionDutchAuctionWithdrawPlan {
    pub fn to_action(&self) -> ActionDutchAuctionWithdraw {
        ActionDutchAuctionWithdraw {
            auction_id: self.auction_id,
            reserves_commitment: self.reserves_commitment(),
            seq: self.seq,
        }
    }

    pub fn reserves_balance(&self) -> Balance {
        Balance::from(self.reserves_input) + Balance::from(self.reserves_output)
    }

    pub fn reserves_commitment(&self) -> balance::Commitment {
        self.reserves_balance().commit(Fr::zero())
    }

    pub fn balance(&self) -> Balance {
        let reserves_balance = self.reserves_balance();
        let prev_auction_nft = Balance::from(Value {
            amount: 1u128.into(),
            asset_id: AuctionNft::new(self.auction_id, self.seq.saturating_sub(1)).asset_id(),
        });

        let next_auction_nft = Balance::from(Value {
            amount: 1u128.into(),
            asset_id: AuctionNft::new(self.auction_id, self.seq).asset_id(),
        });

        reserves_balance + next_auction_nft - prev_auction_nft
    }
}

impl DomainType for ActionDutchAuctionWithdrawPlan {
    type Proto = pb::ActionDutchAuctionWithdrawPlan;
}

impl From<ActionDutchAuctionWithdrawPlan> for pb::ActionDutchAuctionWithdrawPlan {
    fn from(domain: ActionDutchAuctionWithdrawPlan) -> Self {
        Self {
            auction_id: Some(domain.auction_id.into()),
            seq: domain.seq,
            reserves_input: Some(domain.reserves_input.into()),
            reserves_output: Some(domain.reserves_output.into()),
        }
    }
}

impl TryFrom<pb::ActionDutchAuctionWithdrawPlan> for ActionDutchAuctionWithdrawPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::ActionDutchAuctionWithdrawPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            auction_id: msg
                .auction_id
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "ActionDutchAuctionWithdrawPlan message is missing an auction id"
                    )
                })?
                .try_into()?,
            seq: msg.seq,
            reserves_input: msg
                .reserves_input
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "ActionDutchAuctionWithdrawPlan message is missing a reserves input"
                    )
                })?
                .try_into()?,
            reserves_output: msg
                .reserves_output
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "ActionDutchAuctionWithdrawPlan message is missing a reserves output"
                    )
                })?
                .try_into()?,
        })
    }
}
