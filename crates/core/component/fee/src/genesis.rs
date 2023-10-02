use anyhow::Context;
use penumbra_proto::{penumbra::core::component::fee::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

use crate::params::FeeParameters;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial configuration parameters for the fee component.
    pub fee_params: FeeParameters,
}

impl From<Content> for pb::GenesisContent {
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            fee_params: Some(value.fee_params.into()),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            fee_params: msg
                .fee_params
                .context("fee params not present in protobuf message")?
                .try_into()?,
        })
    }
}

impl TypeUrl for Content {
    const TYPE_URL: &'static str = "/penumbra.core.component.fee.v1alpha1.GenesisContent";
}

impl DomainType for Content {
    type Proto = pb::GenesisContent;
}
