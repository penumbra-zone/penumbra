use anyhow::Context;
use penumbra_proto::{penumbra::core::component::dao::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::params::DaoParameters;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial configuration parameters for the DAO component.
    pub dao_params: DaoParameters,
}

impl From<Content> for pb::GenesisContent {
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            dao_params: Some(value.dao_params.into()),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            dao_params: msg
                .dao_params
                .context("dao params not present in protobuf message")?
                .try_into()?,
        })
    }
}

impl DomainType for Content {
    type Proto = pb::GenesisContent;
}
