use anyhow::Context;
use penumbra_proto::{penumbra::core::component::ibc::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::params::IBCParameters;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial configuration parameters for the IBC component.
    pub ibc_params: IBCParameters,
}

impl From<Content> for pb::GenesisContent {
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            ibc_params: Some(value.ibc_params.into()),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            ibc_params: msg
                .ibc_params
                .context("ibc params not present in protobuf message")?
                .try_into()?,
        })
    }
}

impl DomainType for Content {
    type Proto = pb::GenesisContent;
}
