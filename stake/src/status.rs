use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::IdentityKey;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorState", into = "pb::ValidatorState")]
pub enum ValidatorState {
    Inactive,
    Active,
    Unbonding { epoch_index: u64 },
    Slashed,
}

impl Protobuf<pb::ValidatorState> for ValidatorState {}

impl From<ValidatorState> for pb::ValidatorState {
    fn from(msg: ValidatorState) -> Self {
        match msg {
            ValidatorState::Inactive => pb::ValidatorState {
                validator_state: Some(pb::validator_state::ValidatorState::Inactive(
                    "".to_string(),
                )),
            },
            ValidatorState::Active => pb::ValidatorState {
                validator_state: Some(pb::validator_state::ValidatorState::Active("".to_string())),
            },
            ValidatorState::Unbonding { epoch_index } => pb::ValidatorState {
                validator_state: Some(pb::validator_state::ValidatorState::Unbonding {
                    0: epoch_index,
                }),
            },
            ValidatorState::Slashed => pb::ValidatorState {
                validator_state: Some(pb::validator_state::ValidatorState::Slashed("".to_string())),
            },
        }
    }
}

impl TryFrom<pb::ValidatorState> for ValidatorState {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ValidatorState) -> anyhow::Result<Self, Self::Error> {
        if proto.validator_state.is_none() {
            return Err(anyhow::anyhow!("missing validator_state content"));
        }

        match proto.validator_state.unwrap() {
            pb::validator_state::ValidatorState::Inactive(_) => Ok(ValidatorState::Inactive),
            pb::validator_state::ValidatorState::Active(_) => Ok(ValidatorState::Active),
            pb::validator_state::ValidatorState::Unbonding(epoch_index) => {
                Ok(ValidatorState::Unbonding { epoch_index })
            }
            pb::validator_state::ValidatorState::Slashed(_) => Ok(ValidatorState::Slashed),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorStatus", into = "pb::ValidatorStatus")]
pub struct ValidatorStatus {
    pub identity_key: IdentityKey,
    pub voting_power: u64,
    pub state: ValidatorState,
}

impl Protobuf<pb::ValidatorStatus> for ValidatorStatus {}

impl From<ValidatorStatus> for pb::ValidatorStatus {
    fn from(v: ValidatorStatus) -> Self {
        pb::ValidatorStatus {
            identity_key: Some(v.identity_key.into()),
            voting_power: v.voting_power,
        }
    }
}

impl TryFrom<pb::ValidatorStatus> for ValidatorStatus {
    type Error = anyhow::Error;
    fn try_from(v: pb::ValidatorStatus) -> Result<Self, Self::Error> {
        Ok(ValidatorStatus {
            identity_key: v
                .identity_key
                .ok_or_else(|| anyhow::anyhow!("missing identity key field in proto"))?
                .try_into()?,
            voting_power: v.voting_power,
            state: v.state,
        })
    }
}
