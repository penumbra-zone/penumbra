use penumbra_sdk_proto::{penumbra::core::component::stake::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
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
    /// All delegations to the validator will be unbonded at `unbonds_at_height`.
    Unbonding { unbonds_at_height: u64 },
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Bonded => write!(f, "Bonded"),
            State::Unbonded => write!(f, "Unbonded"),
            State::Unbonding { unbonds_at_height } => {
                write!(f, "Unbonding (until height: {unbonds_at_height})")
            }
        }
    }
}

impl DomainType for State {
    type Proto = pb::BondingState;
}

impl From<State> for pb::BondingState {
    #[allow(deprecated)]
    fn from(v: State) -> Self {
        pb::BondingState {
            state: match v {
                State::Bonded => pb::bonding_state::BondingStateEnum::Bonded as i32,
                State::Unbonded => pb::bonding_state::BondingStateEnum::Unbonded as i32,
                State::Unbonding { .. } => pb::bonding_state::BondingStateEnum::Unbonding as i32,
            },
            unbonds_at_epoch: 0,
            unbonds_at_height: match v {
                State::Unbonding { unbonds_at_height } => unbonds_at_height,
                _ => 0,
            },
        }
    }
}

impl TryFrom<pb::BondingState> for State {
    type Error = anyhow::Error;
    fn try_from(v: pb::BondingState) -> Result<Self, Self::Error> {
        let bonding_state = pb::bonding_state::BondingStateEnum::try_from(v.state)
            .map_err(|e| anyhow::anyhow!("invalid bonding state, error: {e}"))?;

        match bonding_state {
            pb::bonding_state::BondingStateEnum::Bonded => Ok(State::Bonded),
            pb::bonding_state::BondingStateEnum::Unbonded => Ok(State::Unbonded),
            pb::bonding_state::BondingStateEnum::Unbonding => {
                let unbonds_at_height = if v.unbonds_at_height > 0 {
                    v.unbonds_at_height
                } else {
                    anyhow::bail!("unbonding epoch should be set for unbonding state")
                };
                Ok(State::Unbonding { unbonds_at_height })
            }
            pb::bonding_state::BondingStateEnum::Unspecified => {
                Err(anyhow::anyhow!("unspecified bonding state!"))
            }
        }
    }
}
