use std::str::FromStr;

use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::{IdentityKey, ValidatorState};

/// The current status of a validator, including its identity, voting power, and state in the
/// validator state machine.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorStatus", into = "pb::ValidatorStatus")]
pub struct ValidatorStatus {
    /// The validator's identity.
    pub identity_key: IdentityKey,
    /// The validator's voting power.
    pub voting_power: u64,
    /// The validator's current state.
    pub state: ValidatorState,
}

impl Protobuf<pb::ValidatorStatus> for ValidatorStatus {}

impl From<ValidatorStatus> for pb::ValidatorStatus {
    fn from(v: ValidatorStatus) -> Self {
        pb::ValidatorStatus {
            identity_key: Some(v.identity_key.into()),
            voting_power: v.voting_power,
            state: match v.state {
                ValidatorState::Inactive => pb::validator_status::ValidatorState::Inactive,
                ValidatorState::Active => pb::validator_status::ValidatorState::Active,
                ValidatorState::Unbonding { .. } => pb::validator_status::ValidatorState::Unbonding,
                ValidatorState::Slashed => pb::validator_status::ValidatorState::Slashed,
            } as i32,
            unbonding_epoch: match v.state {
                ValidatorState::Unbonding { unbonding_epoch } => Some(unbonding_epoch),
                _ => None,
            },
        }
    }
}

impl TryFrom<pb::ValidatorStatus> for ValidatorStatus {
    type Error = anyhow::Error;
    fn try_from(v: pb::ValidatorStatus) -> Result<Self, Self::Error> {
        let state = match pb::validator_status::ValidatorState::from_i32(v.state)
            .ok_or_else(|| anyhow::anyhow!("missing validator state"))?
        {
            pb::validator_status::ValidatorState::Inactive => ValidatorState::Inactive,
            pb::validator_status::ValidatorState::Active => ValidatorState::Active,
            pb::validator_status::ValidatorState::Unbonding => ValidatorState::Unbonding {
                unbonding_epoch: v
                    .unbonding_epoch
                    .ok_or_else(|| anyhow::anyhow!("missing unbonding epoch"))?,
            },
            pb::validator_status::ValidatorState::Slashed => ValidatorState::Slashed,
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
