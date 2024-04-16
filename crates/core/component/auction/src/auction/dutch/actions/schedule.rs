use crate::auction::{dutch::DutchAuctionDescription, nft::AuctionNft};
use anyhow::anyhow;
use penumbra_asset::{Balance, Value};
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

impl ActionDutchAuctionSchedule {
    /// Compute the value balance corresponding to this action:
    ///
    /// # Diagram
    ///
    ///  ┌────────────────────┬──────────────────────┐
    ///  │      Burn (-)      │       Mint (+)       │
    ///  ├────────────────────┼──────────────────────┤
    ///  │    input value     │  opened auction nft  │
    ///  └────────────────────┴──────────────────────┘                  
    pub fn balance(&self) -> Balance {
        let opened_auction_nft = AuctionNft::new(self.description.id(), 0u64);
        let opened_auction_nft_value = Value {
            asset_id: opened_auction_nft.metadata.id(),
            amount: 1u128.into(),
        };

        let output_nft_balance = Balance::from(opened_auction_nft_value);
        let input_balance = Balance::from(self.description.input);

        output_nft_balance - input_balance
    }
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
