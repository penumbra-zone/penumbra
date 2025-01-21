use crate::auction::{
    dutch::{
        actions::{ActionDutchAuctionSchedule, ActionDutchAuctionWithdraw},
        asset::Metadata,
    },
    id::AuctionId,
};
use anyhow::anyhow;
use penumbra_sdk_asset::ValueView;
use penumbra_sdk_proto::{core::component::auction::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/* Domain type definitions */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ActionDutchAuctionScheduleView",
    into = "pb::ActionDutchAuctionScheduleView"
)]
pub struct ActionDutchAuctionScheduleView {
    pub action: ActionDutchAuctionSchedule,
    pub auction_id: AuctionId,
    pub input_metadata: Option<Metadata>,
    pub output_metadata: Option<Metadata>,
}

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

/* Conversion back to an action */

impl From<ActionDutchAuctionScheduleView> for ActionDutchAuctionSchedule {
    fn from(value: ActionDutchAuctionScheduleView) -> Self {
        value.action
    }
}

impl From<ActionDutchAuctionWithdrawView> for ActionDutchAuctionWithdraw {
    fn from(value: ActionDutchAuctionWithdrawView) -> Self {
        value.action
    }
}

/* Protobuf impls */
impl DomainType for ActionDutchAuctionScheduleView {
    type Proto = pb::ActionDutchAuctionScheduleView;
}

impl From<ActionDutchAuctionScheduleView> for pb::ActionDutchAuctionScheduleView {
    fn from(domain: ActionDutchAuctionScheduleView) -> Self {
        pb::ActionDutchAuctionScheduleView {
            action: Some(domain.action.into()),
            auction_id: Some(domain.auction_id.into()),
            input_metadata: domain.input_metadata.map(Into::into),
            output_metadata: domain.output_metadata.map(Into::into),
        }
    }
}

impl TryFrom<pb::ActionDutchAuctionScheduleView> for ActionDutchAuctionScheduleView {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ActionDutchAuctionScheduleView) -> Result<Self, Self::Error> {
        Ok(ActionDutchAuctionScheduleView {
            action: msg
                .action
                .ok_or_else(|| {
                    anyhow!("ActionDutchAuctionScheduleView message is missing an action")
                })?
                .try_into()?,
            auction_id: msg
                .auction_id
                .ok_or_else(|| {
                    anyhow!("ActionDutchAuctionScheduleView message is missing an auction_id")
                })?
                .try_into()?,
            input_metadata: msg
                .input_metadata
                .map(|input| input.try_into())
                .transpose()?,
            output_metadata: msg
                .output_metadata
                .map(|output| output.try_into())
                .transpose()?,
        })
    }
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
