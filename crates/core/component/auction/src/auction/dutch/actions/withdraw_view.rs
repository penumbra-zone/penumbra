use crate::auction::dutch::actions::ActionDutchAuctionWithdraw;
use anyhow::anyhow;
use penumbra_asset::ValueView;
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
            reserves: domain
                .reserves
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
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
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}
