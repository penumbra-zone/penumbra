use decaf377::{FieldExt, Fr};
use penumbra_proto::{transaction as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::action::ValidatorVoteBody;

/// A plan to vote as a delegator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorVotePlan", into = "pb::ValidatorVotePlan")]
pub struct ValidatorVotePlan {
    /// The body of the proposal withdrawal.
    pub body: ValidatorVoteBody,
    /// The randomizer to use for the signature.
    pub randomizer: Fr,
}

impl Protobuf<pb::ValidatorVotePlan> for ValidatorVotePlan {}

impl From<ValidatorVotePlan> for pb::ValidatorVotePlan {
    fn from(inner: ValidatorVotePlan) -> Self {
        pb::ValidatorVotePlan {
            body: Some(inner.body.into()),
            randomizer: inner.randomizer.to_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<pb::ValidatorVotePlan> for ValidatorVotePlan {
    type Error = anyhow::Error;

    fn try_from(value: pb::ValidatorVotePlan) -> Result<Self, Self::Error> {
        Ok(ValidatorVotePlan {
            body: value
                .body
                .ok_or_else(|| anyhow::anyhow!("missing body in `ValidatorVotePlan`"))?
                .try_into()?,
            randomizer: Fr::from_bytes(value.randomizer.as_ref().try_into()?)?,
        })
    }
}
