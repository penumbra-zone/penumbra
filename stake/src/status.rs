use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::IdentityKey;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorState", into = "pb::ValidatorState")]
pub enum ValidatorState {
    Inactive,
    Active,
    Unbonding,
    Slashed,
}

impl Protobuf<pb::validator_status::ValidatorState> for ValidatorState {}

impl From<ValidatorState> for pb::validator_status::ValidatorState {
    fn from(msg: ValidatorState) -> Self {
        match msg {
            ValidatorState::Inactive => pb::validator_status::ValidatorState {
                validator_state: Some(pb::validator_status::ValidatorState::Inactive),
            },
            ValidatorState::Active => pb::validator_status::ValidatorState {
                validator_state: Some(pb::validator_status::ValidatorState::Active),
            },
            ValidatorState::Unbonding => pb::validator_status::ValidatorState {
                validator_state: Some(pb::validator_status::ValidatorState::Unbonding),
            },
            ValidatorState::Slashed => pb::validator_status::ValidatorState {
                validator_state: Some(pb::validator_status::ValidatorState::Slashed),
            },
        }
    }
}

impl TryFrom<pb::validator_status::ValidatorState> for ValidatorState {
    type Error = anyhow::Error;

    fn try_from(proto: pb::validator_status::ValidatorState) -> anyhow::Result<Self, Self::Error> {
        if proto.validator_state.is_none() {
            return Err(anyhow::anyhow!("missing validator_state content"));
        }

        match proto.validator_state.unwrap() {
            pb::validator_status::ValidatorState::Inactive => Ok(ValidatorState::Inactive),
            pb::validator_status::ValidatorState::Active => Ok(ValidatorState::Active),
            pb::validator_status::ValidatorState::Unbonding => Ok(ValidatorState::Unbonding),
            pb::validator_status::ValidatorState::Slashed => Ok(ValidatorState::Slashed),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorStatus", into = "pb::ValidatorStatus")]
pub struct ValidatorStatus {
    pub identity_key: IdentityKey,
    pub voting_power: u64,
    pub state: ValidatorState,
    pub unbonding_epoch: Option<u64>,
}

impl Protobuf<pb::ValidatorStatus> for ValidatorStatus {}

impl From<ValidatorStatus> for pb::ValidatorStatus {
    fn from(v: ValidatorStatus) -> Self {
        pb::ValidatorStatus {
            identity_key: Some(v.identity_key.into()),
            voting_power: v.voting_power,
            state: Some(v.state.into()),
            unbonding_epoch: v.unbonding_epoch,
        }
    }
}

impl TryFrom<pb::ValidatorStatus> for ValidatorStatus {
    type Error = anyhow::Error;
    fn try_from(v: pb::ValidatorStatus) -> Result<Self, Self::Error> {
        let state = match v.state.unwrap() {
            pb::ValidatorState { validator_state } => match validator_state.unwrap() {
                pb::validator_state::ValidatorState::Inactive => ValidatorState::Inactive,
                pb::validator_state::ValidatorState::Active => ValidatorState::Active,
                pb::validator_state::ValidatorState::Unbonding => {
                    ValidatorState::Unbonding { epoch_index }
                }
                pb::validator_state::ValidatorState::Slashed => ValidatorState::Slashed,
            },
        };

        Ok(ValidatorStatus {
            identity_key: v
                .identity_key
                .ok_or_else(|| anyhow::anyhow!("missing identity key field in proto"))?
                .try_into()?,
            voting_power: v.voting_power,
            state,
        })
    }
}
