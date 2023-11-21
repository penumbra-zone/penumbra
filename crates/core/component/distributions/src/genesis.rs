use anyhow::Context;
use penumbra_proto::{penumbra::core::component::distributions::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::params::DistributionsParameters;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial configuration parameters for the distributions component.
    pub distributions_params: DistributionsParameters,
}

impl From<Content> for pb::GenesisContent {
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            distributions_params: Some(value.distributions_params.into()),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            distributions_params: msg
                .distributions_params
                .context("distributions params not present in protobuf message")?
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
            distributions_params: DistributionsParameters::default(),
        }
    }
}
