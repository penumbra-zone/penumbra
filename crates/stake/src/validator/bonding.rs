use penumbra_proto::{core::stake::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::BondingState", into = "pb::BondingState")]
pub enum State {
    /// The validator is in the active set.
    ///
    /// All stake to the validator is bonded.
    ///
    /// Undelegations must wait for an unbonding period to pass before
    /// their undelegation completes.
    Bonded,
    /// The validator is not in the active set.
    ///
    /// Delegations to the validator are unbonded and free to be undelegated at
    /// any time.
    Unbonded,
    /// The validator has been removed from the active set.
    ///
    /// All delegations to the validator will be unbonded at `unbonding_epoch`.
    Unbonding { unbonding_epoch: u64 },
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Bonded => write!(f, "Bonded"),
            State::Unbonded => write!(f, "Unbonded"),
            State::Unbonding { unbonding_epoch } => {
                write!(f, "Unbonding (end epoch: {unbonding_epoch})")
            }
        }
    }
}

impl DomainType for State {
    type Proto = pb::BondingState;
}

impl From<State> for pb::BondingState {
    fn from(v: State) -> Self {
        pb::BondingState {
            state: match v {
                State::Bonded => pb::bonding_state::BondingStateEnum::Bonded as i32,
                State::Unbonded => pb::bonding_state::BondingStateEnum::Unbonded as i32,
                State::Unbonding { unbonding_epoch: _ } => {
                    pb::bonding_state::BondingStateEnum::Unbonding as i32
                }
            },
            unbonding_epoch: match v {
                State::Unbonding { unbonding_epoch } => Some(unbonding_epoch),
                _ => None,
            },
        }
    }
}

impl TryFrom<pb::BondingState> for State {
    type Error = anyhow::Error;
    fn try_from(v: pb::BondingState) -> Result<Self, Self::Error> {
        let Some(bonding_state) = pb::bonding_state::BondingStateEnum::from_i32(v.state) else {
            return Err(anyhow::anyhow!("invalid bonding state!"))
        };

        match bonding_state {
            pb::bonding_state::BondingStateEnum::Bonded => Ok(State::Bonded),
            pb::bonding_state::BondingStateEnum::Unbonded => Ok(State::Unbonded),
            pb::bonding_state::BondingStateEnum::Unbonding => {
                let Some(unbonding_epoch) = v.unbonding_epoch else {
            return Err(anyhow::anyhow!("unbonding epoch should be set for unbonding state"))
        };
                Ok(State::Unbonding { unbonding_epoch })
            }
            pb::bonding_state::BondingStateEnum::Unspecified => {
                Err(anyhow::anyhow!("unspecified bonding state!"))
            }
        }
    }
}
