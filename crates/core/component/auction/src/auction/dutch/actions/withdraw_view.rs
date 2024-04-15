use crate::auction::dutch::DutchAuctionauction_id;
use anyhow::anyhow;
use penumbra_proto::{core::component::auction::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ActionDutchAuctionWithdrawView",
    into = "pb::ActionDutchAuctionWithdrawView"
)]
pub struct ActionDutchAuctionWithdrawView {
    pub action: ActionDutchAuctionWithdraw,
    // A sequence of values that sum together to the provided
    // reserves commitment.
    pub reserves: Vec<ValueView>,
}

/* Protobuf impls */
impl DomainType for ActionDutchAuctionWithdrawView {
    type Proto = pb::ActionDutchAuctionWithdrawView;
}

impl From<ActionDutchAuctionWithdrawView> for pb::ActionDutchAuctionWithdrawView {
    fn from(domain: ActionDutchAuctionWithdrawView) -> Self {
        pb::ActionDutchAuctionWithdrawView {
            action: Some(domain.action.into()),
            reserves: Some(domain.reserves.into()),
        }
    }
}

impl TryFrom<pb::ActionDutchAuctionWithdrawView> for ActionDutchAuctionWithdrawView {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ActionDutchAuctionWithdrawView) -> Result<Self, Self::Error> {
        Ok(ActionDutchAuctionWithdrawView {
            action: msg
                .action
                .ok_or_else(|| {
                    anyhow!("ActionDutchAuctionWithdrawView message is missing an action")
                })?
                .try_into()?,
            reserves: msg
                .reserves
                .ok_or_else(|| {
                    anyhow!("ActionDutchAuctionWithdrawView message is missing reserves")
                })?
                .try_into()?,
        })
    }
}
