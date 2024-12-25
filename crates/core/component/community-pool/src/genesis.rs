use anyhow::Context;
use penumbra_sdk_asset::{Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_proto::{penumbra::core::component::community_pool::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::params::CommunityPoolParameters;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial configuration parameters for the Community Pool component.
    pub community_pool_params: CommunityPoolParameters,
    /// The initial balance of the Community Pool.
    pub initial_balance: Value,
}

impl From<Content> for pb::GenesisContent {
    fn from(genesis: Content) -> Self {
        pb::GenesisContent {
            community_pool_params: Some(genesis.community_pool_params.into()),
            initial_balance: Some(genesis.initial_balance.into()),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            initial_balance: msg
                .initial_balance
                .context("Initial balance not present in protobuf message")?
                .try_into()?,
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

impl Default for Content {
    fn default() -> Self {
        Content {
            community_pool_params: CommunityPoolParameters::default(),
            initial_balance: Value {
                amount: 0u128.into(),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
        }
    }
}
