use crate::auction::{
    dutch::{actions::ActionDutchAuctionSchedule, asset::Metadata},
    id::AuctionId,
};
use anyhow::anyhow;
use penumbra_proto::{core::component::auction::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ActionDutchAuctionScheduleView",
    into = "pb::ActionDutchAuctionScheduleView"
)]
pub struct ActionDutchAuctionScheduleView {
    pub action: ActionDutchAuctionSchedule,
    pub auction_id: AuctionId,
    pub input_metadata: Metadata,
    pub output_metadata: Metadata,
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
            input_metadata: Some(domain.input_metadata.into()),
            output_metadata: Some(domain.output_metadata.into()),
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
                .ok_or_else(|| {
                    anyhow!("ActionDutchAuctionScheduleView message is missing an input_metadata")
                })?
                .try_into()?,
            output_metadata: msg
                .output_metadata
                .ok_or_else(|| {
                    anyhow!("ActionDutchAuctionScheduleView message is missing an output_metadata")
                })?
                .try_into()?,
        })
    }
}
