use crate::params::AuctionParameters;
use anyhow::Context;
use serde::{Deserialize, Serialize};

use penumbra_sdk_proto::{penumbra::core::component::auction::v1 as pb, DomainType};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial configuration parameters for the auction component.
    pub auction_params: AuctionParameters,
}

impl DomainType for Content {
    type Proto = pb::GenesisContent;
}

impl From<Content> for pb::GenesisContent {
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            params: Some(value.auction_params.into()),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            auction_params: msg
                .params
                .context("auction params not present in protobuf message")?
                .try_into()?,
        })
    }
}
