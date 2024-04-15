use crate::auction::dutch::DutchAuctionDescription;
use anyhow::anyhow;
use penumbra_proto::{core::component::auction::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ActionDutchAuctionSchedule",
    into = "pb::ActionDutchAuctionSchedule"
)]
pub struct ActionDutchAuctionSchedule {
    pub description: DutchAuctionDescription,
}

/* Protobuf impls */
impl DomainType for ActionDutchAuctionSchedule {
    type Proto = pb::ActionDutchAuctionSchedule;
}

impl From<ActionDutchAuctionSchedule> for pb::ActionDutchAuctionSchedule {
    fn from(domain: ActionDutchAuctionSchedule) -> Self {
        pb::ActionDutchAuctionSchedule {
            description: Some(domain.description.into()),
        }
    }
}

impl TryFrom<pb::ActionDutchAuctionSchedule> for ActionDutchAuctionSchedule {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ActionDutchAuctionSchedule) -> Result<Self, Self::Error> {
        Ok(ActionDutchAuctionSchedule {
            description: msg
                .description
                .ok_or_else(|| {
                    anyhow!("ActionDutchAuctionSchedule message is missing a description")
                })?
                .try_into()?,
        })
    }
}
