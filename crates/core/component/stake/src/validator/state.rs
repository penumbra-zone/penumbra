use anyhow::anyhow;
use penumbra_sdk_proto::{penumbra::core::component::stake::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// The state of a validator in the validator state machine.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorState", into = "pb::ValidatorState")]
pub enum State {
    /// The validator definition has been published, but the validator stake is below the minimum
    /// threshold required to be indexed.
    Defined,
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
            State::Defined => write!(f, "Defined"),
            State::Inactive => write!(f, "Inactive"),
            State::Active => write!(f, "Active"),
            State::Jailed => write!(f, "Jailed"),
            State::Tombstoned => write!(f, "Tombstoned"),
            State::Disabled => write!(f, "Disabled"),
        }
    }
}

impl DomainType for State {
    type Proto = pb::ValidatorState;
}

impl From<State> for pb::ValidatorState {
    fn from(v: State) -> Self {
        pb::ValidatorState {
            state: match v {
                State::Defined => pb::validator_state::ValidatorStateEnum::Defined,
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
        let validator_state = pb::validator_state::ValidatorStateEnum::try_from(v.state)
            .map_err(|e| anyhow::anyhow!("invalid validator state, error: {e}"))?;

        match validator_state {
            pb::validator_state::ValidatorStateEnum::Defined => Ok(State::Defined),
            pb::validator_state::ValidatorStateEnum::Inactive => Ok(State::Inactive),
            pb::validator_state::ValidatorStateEnum::Active => Ok(State::Active),
            pb::validator_state::ValidatorStateEnum::Jailed => Ok(State::Jailed),
            pb::validator_state::ValidatorStateEnum::Tombstoned => Ok(State::Tombstoned),
            pb::validator_state::ValidatorStateEnum::Disabled => Ok(State::Disabled),
            pb::validator_state::ValidatorStateEnum::Unspecified => {
                Err(anyhow!("unspecified validator state!"))
            }
        }
    }
}
