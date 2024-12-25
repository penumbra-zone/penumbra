use anyhow::Context;
use penumbra_sdk_proto::{penumbra::core::component::dex::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::DexParameters;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial configuration parameters for the DEX component.
    pub dex_params: DexParameters,
}

impl From<Content> for pb::GenesisContent {
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            dex_params: Some(value.dex_params.into()),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            dex_params: msg
                .dex_params
                .context("dex params not present in protobuf message")?
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
            dex_params: DexParameters::default(),
        }
    }
}
