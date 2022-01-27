use std::str::FromStr;

use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::IdentityKey;

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

/// The state of a validator in the validator state machine.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValidatorState {
    /// The validator is not currently a part of the consensus set, but could become so if it
    /// acquired enough voting power.
    Inactive,
    /// The validator is an active part of the consensus set.
    Active,
    /// The validator has been removed from the consensus set, and all stake will finish unbonding
    /// at the epoch `unbonding_epoch`.
    Unbonding { unbonding_epoch: u64 },
    /// The validator has been slashed, and undelegations will occur immediately with no unbonding
    /// period.
    Slashed,
}

/// The name of a validator state, as a "C-style enum" without the extra information such as the
/// `unbonding_epoch`.
pub enum ValidatorStateName {
    /// The state name for [`ValidatorState::Inactive`].
    Inactive,
    /// The state name for [`ValidatorState::Active`].
    Active,
    /// The state name for [`ValidatorState::Unbonding`].
    Unbonding,
    /// The state name for [`ValidatorState::Slashed`].
    Slashed,
}

impl ValidatorState {
    /// Returns the name of the validator state.
    pub fn name(&self) -> ValidatorStateName {
        match self {
            ValidatorState::Inactive => ValidatorStateName::Inactive,
            ValidatorState::Active => ValidatorStateName::Active,
            ValidatorState::Unbonding { .. } => ValidatorStateName::Unbonding,
            ValidatorState::Slashed => ValidatorStateName::Slashed,
        }
    }
}

impl ValidatorStateName {
    /// Returns a static string representation of the validator state name.
    ///
    /// This is stable and should be used when serializing to strings (it is the inverse of [`FromStr::from_str`]).
    pub fn to_str(&self) -> &'static str {
        match self {
            ValidatorStateName::Inactive => "INACTIVE",
            ValidatorStateName::Active => "ACTIVE",
            ValidatorStateName::Unbonding => "UNBONDING",
            ValidatorStateName::Slashed => "SLASHED",
        }
    }
}

impl FromStr for ValidatorStateName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INACTIVE" => Ok(ValidatorStateName::Inactive),
            "ACTIVE" => Ok(ValidatorStateName::Active),
            "UNBONDING" => Ok(ValidatorStateName::Unbonding),
            "SLASHED" => Ok(ValidatorStateName::Slashed),
            _ => Err(anyhow::anyhow!("invalid validator state name: {}", s)),
        }
    }
}

impl From<ValidatorState> for (ValidatorStateName, Option<u64>) {
    fn from(state: ValidatorState) -> Self {
        match state {
            ValidatorState::Inactive => (ValidatorStateName::Inactive, None),
            ValidatorState::Active => (ValidatorStateName::Active, None),
            ValidatorState::Unbonding { unbonding_epoch } => {
                (ValidatorStateName::Unbonding, Some(unbonding_epoch))
            }
            ValidatorState::Slashed => (ValidatorStateName::Slashed, None),
        }
    }
}

impl TryFrom<(ValidatorStateName, Option<u64>)> for ValidatorState {
    type Error = anyhow::Error;

    fn try_from(state: (ValidatorStateName, Option<u64>)) -> Result<Self, Self::Error> {
        match state {
            (ValidatorStateName::Inactive, None) => Ok(ValidatorState::Inactive),
            (ValidatorStateName::Active, None) => Ok(ValidatorState::Active),
            (ValidatorStateName::Unbonding, Some(unbonding_epoch)) => {
                Ok(ValidatorState::Unbonding { unbonding_epoch })
            }
            (ValidatorStateName::Slashed, None) => Ok(ValidatorState::Slashed),
            (_, Some(_)) => Err(anyhow::anyhow!(
                "unbonding epoch not permitted with non-unbonding state"
            )),
            (ValidatorStateName::Unbonding, None) => Err(anyhow::anyhow!(
                "unbonding epoch not provided with unbonding state"
            )),
        }
    }
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
