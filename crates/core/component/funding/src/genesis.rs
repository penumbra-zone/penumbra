use anyhow::Context;
use penumbra_sdk_proto::{penumbra::core::component::funding::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::params::FundingParameters;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial configuration parameters for the Funding component.
    pub funding_params: FundingParameters,
}

impl From<Content> for pb::GenesisContent {
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            funding_params: Some(value.funding_params.into()),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            funding_params: msg
                .funding_params
                .context("Funding params not present in protobuf message")?
                .try_into()?,
        })
    }
}

impl DomainType for Content {
    type Proto = pb::GenesisContent;
}

impl Default for Content {
    fn default() -> Self {
        Self {
            funding_params: FundingParameters::default(),
        }
    }
}
