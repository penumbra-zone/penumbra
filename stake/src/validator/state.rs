use penumbra_proto::{stake as pb, Protobuf};
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
    /// The validator has been slashed, and undelegations will occur immediately with no unbonding
    /// period.
    Slashed,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Inactive => write!(f, "Inactive"),
            State::Active => write!(f, "Active"),
            State::Slashed => write!(f, "Slashed"),
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
                State::Slashed => pb::validator_state::ValidatorStateEnum::Slashed,
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
                pb::validator_state::ValidatorStateEnum::Slashed => State::Slashed,
            },
        )
    }
}
