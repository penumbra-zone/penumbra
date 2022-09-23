use penumbra_proto::{core::stake::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};

use super::IdentityKey;

/// A list of validators.
///
/// This is a newtype wrapper for a Vec that allows us to define a proto type.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorList", into = "pb::ValidatorList")]
pub struct List(pub Vec<IdentityKey>);

impl Protobuf<pb::ValidatorList> for List {}

impl TryFrom<pb::ValidatorList> for List {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ValidatorList) -> Result<Self, Self::Error> {
        Ok(List(
            msg.validator_keys
                .iter()
                .map(|key| key.clone().try_into())
                .collect::<anyhow::Result<Vec<_>>>()?,
        ))
    }
}

impl From<List> for pb::ValidatorList {
    fn from(vk: List) -> Self {
        pb::ValidatorList {
            validator_keys: vk.0.iter().map(|v| v.clone().into()).collect(),
        }
    }
}
