use anyhow::Context;
use penumbra_sdk_proto::{penumbra::core::component::stake::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::params::StakeParameters;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial configuration parameters for the staking component.
    pub stake_params: StakeParameters,
    /// The initial validator set.
    pub validators: Vec<pb::Validator>,
}

impl DomainType for Content {
    type Proto = pb::GenesisContent;
}

impl From<Content> for pb::GenesisContent {
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            stake_params: Some(value.stake_params.into()),
            validators: value.validators.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            stake_params: msg
                .stake_params
                .context("stake params not present in protobuf message")?
                .try_into()?,
            validators: msg
                .validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}
