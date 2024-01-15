use anyhow::Context;
use penumbra_proto::{penumbra::core::component::community_pool::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::params::CommunityPoolParameters;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial configuration parameters for the Community Pool component.
    pub community_pool_params: CommunityPoolParameters,
}

impl From<Content> for pb::GenesisContent {
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            community_pool_params: Some(value.community_pool_params.into()),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            community_pool_params: msg
                .community_pool_params
                .context("Community Pool params not present in protobuf message")?
                .try_into()?,
        })
    }
}

impl DomainType for Content {
    type Proto = pb::GenesisContent;
}
