use crate::auction::dutch::DutchAuctionDescription;
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
