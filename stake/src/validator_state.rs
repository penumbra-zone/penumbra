use std::str::FromStr;

use serde::{Deserialize, Serialize};

use penumbra_proto::{stake as pb, Protobuf};

/// The state of a validator in the validator state machine.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorState", into = "pb::ValidatorState")]
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

impl std::fmt::Display for ValidatorState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ValidatorState::Inactive => write!(f, "Inactive"),
            ValidatorState::Active => write!(f, "Active"),
            ValidatorState::Unbonding { unbonding_epoch } => {
                write!(f, "Unbonding (unbonding epoch: {})", unbonding_epoch)
            }
            ValidatorState::Slashed => write!(f, "Slashed"),
        }
    }
}

impl Protobuf<pb::ValidatorState> for ValidatorState {}

impl From<ValidatorState> for pb::ValidatorState {
    fn from(v: ValidatorState) -> Self {
        pb::ValidatorState {
            unbonding_epoch: match v {
                ValidatorState::Unbonding { unbonding_epoch } => Some(unbonding_epoch),
                _ => None,
            },
            state: match v {
                ValidatorState::Inactive => pb::validator_state::ValidatorStateEnum::Inactive,
                ValidatorState::Active => pb::validator_state::ValidatorStateEnum::Active,
                ValidatorState::Unbonding { .. } => {
                    pb::validator_state::ValidatorStateEnum::Unbonding
                }
                ValidatorState::Slashed => pb::validator_state::ValidatorStateEnum::Slashed,
            } as i32,
        }
    }
}

impl TryFrom<pb::ValidatorState> for ValidatorState {
    type Error = anyhow::Error;
    fn try_from(v: pb::ValidatorState) -> Result<Self, Self::Error> {
        Ok(
            match pb::validator_state::ValidatorStateEnum::from_i32(v.state)
                .ok_or_else(|| anyhow::anyhow!("missing validator state"))?
            {
                pb::validator_state::ValidatorStateEnum::Inactive => ValidatorState::Inactive,
                pb::validator_state::ValidatorStateEnum::Active => ValidatorState::Active,
                pb::validator_state::ValidatorStateEnum::Unbonding => ValidatorState::Unbonding {
                    unbonding_epoch: v
                        .unbonding_epoch
                        .ok_or_else(|| anyhow::anyhow!("missing unbonding epoch"))?,
                },
                pb::validator_state::ValidatorStateEnum::Slashed => ValidatorState::Slashed,
            },
        )
    }
}

/// The name of a validator state, as a "C-style enum" without the extra information such as the
/// `unbonding_epoch`.
/// TODO: is this necessary now? Is `ValidatorStatus` necessary or are we decomposing into different
/// paths in the JMT?
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
