use anyhow::Context;
use penumbra_proto::{penumbra::core::component::governance::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

use crate::params::GovernanceParameters;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial configuration parameters for the governance component.
    pub governance_params: GovernanceParameters,
}

impl From<Content> for pb::GenesisContent {
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            governance_params: Some(value.governance_params.into()),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            governance_params: msg
                .governance_params
                .context("governance params not present in protobuf message")?
                .try_into()?,
        })
    }
}

impl TypeUrl for Content {
    const TYPE_URL: &'static str = "/penumbra.governance.v1alpha1.GenesisContent";
}

impl DomainType for Content {
    type Proto = pb::GenesisContent;
}
