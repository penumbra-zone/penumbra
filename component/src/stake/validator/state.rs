use penumbra_proto::{core::stake::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};

/// The state of a validator in the validator state machine.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorState", into = "pb::ValidatorState")]
pub enum State {
    /// The validator is not currently a part of the consensus set, but could become so if it
    /// acquired enough voting power.
    Inactive,
    /// The validator is an active part of the consensus set.
    Active,
    /// The validator has been slashed for downtime, and is prevented from participation
    /// in consensus until it requests to be reinstated.
    Jailed,
    /// The validator has been slashed for byzantine misbehavior, and is permanently banned.
    Tombstoned,
    /// The validator operator has disabled this validator's operations.
    ///
    /// Delegations to this validator are not allowed.
    Disabled,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Inactive => write!(f, "Inactive"),
            State::Active => write!(f, "Active"),
            State::Jailed => write!(f, "Jailed"),
            State::Tombstoned => write!(f, "Tombstoned"),
            State::Disabled => write!(f, "Disabled"),
        }
    }
}

impl Protobuf<pb::ValidatorState> for State {}

impl From<State> for pb::ValidatorState {
    fn from(v: State) -> Self {
        pb::ValidatorState {
            state: match v {
                State::Inactive => pb::validator_state::ValidatorStateEnum::Inactive,
                State::Active => pb::validator_state::ValidatorStateEnum::Active,
                State::Jailed => pb::validator_state::ValidatorStateEnum::Jailed,
                State::Tombstoned => pb::validator_state::ValidatorStateEnum::Tombstoned,
                State::Disabled => pb::validator_state::ValidatorStateEnum::Disabled,
            } as i32,
        }
    }
}

impl TryFrom<pb::ValidatorState> for State {
    type Error = anyhow::Error;
    fn try_from(v: pb::ValidatorState) -> Result<Self, Self::Error> {
        Ok(
            match pb::validator_state::ValidatorStateEnum::from_i32(v.state)
                .ok_or_else(|| anyhow::anyhow!("missing validator state"))?
            {
                pb::validator_state::ValidatorStateEnum::Inactive => State::Inactive,
                pb::validator_state::ValidatorStateEnum::Active => State::Active,
                pb::validator_state::ValidatorStateEnum::Jailed => State::Jailed,
                pb::validator_state::ValidatorStateEnum::Tombstoned => State::Tombstoned,
                pb::validator_state::ValidatorStateEnum::Disabled => State::Disabled,
            },
        )
    }
}
