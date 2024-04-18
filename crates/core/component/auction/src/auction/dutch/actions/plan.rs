use ark_ff::Zero;
use decaf377::Fr;
use penumbra_asset::{balance, Balance, Value};
use penumbra_proto::{penumbra::core::component::auction::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::auction::{dutch::ActionDutchAuctionWithdraw, AuctionId};

/// A planned [`ActionDutchAuctionWithdraw`](ActionDutchAuctionWithdraw).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    try_from = "pb::ActionDutchAuctionWithdrawPlan",
    into = "pb::ActionDutchAuctionWithdrawPlan"
)]
pub struct ActionDutchAuctionWithdrawPlan {
    pub auction_id: AuctionId,
    pub seq: u64,
    pub reserves: Balance,
}

impl ActionDutchAuctionWithdrawPlan {
    /// Convenience method to construct the [`ActionDutchAuctionWithdraw`] described by this [`ActionDutchAuctionWithdrawPlan`].
    pub fn ActionDutchAuction_withdraw(&self) -> ActionDutchAuctionWithdraw {
        ActionDutchAuctionWithdraw {
            auction_id: self.auction_id,
            reserves_commitment: self.reserves_commitment(),
            seq: self.seq,
        }
    }

    pub fn reserves_commitment(&self) -> balance::Commitment {
        self.reserves.commit(Fr::zero())
    }

    pub fn balance(&self) -> Balance {
        let mut balance = self.reserves.balance(&self.pair);
        todo!()
    }
}

impl DomainType for ActionDutchAuctionWithdrawPlan {
    type Proto = pb::ActionDutchAuctionWithdrawPlan;
}

impl From<ActionDutchAuctionWithdrawPlan> for pb::ActionDutchAuctionWithdrawPlan {
    fn from(msg: ActionDutchAuctionWithdrawPlan) -> Self {
        todo!()
    }
}

impl TryFrom<pb::ActionDutchAuctionWithdrawPlan> for ActionDutchAuctionWithdrawPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::ActionDutchAuctionWithdrawPlan) -> Result<Self, Self::Error> {
        todo!()
    }
}
