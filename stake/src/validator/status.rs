use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::{validator::State, IdentityKey};

/// The current status of a validator, including its identity, voting power, and state in the
/// validator state machine.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorStatus", into = "pb::ValidatorStatus")]
pub struct Status {
    /// The validator's identity.
    pub identity_key: IdentityKey,
    /// The validator's voting power. Note that only `Active` validators are part of the consensus set
    /// and will have their voting power returned to Tendermint. Non-`Active` validators will return
    /// voting power 0 to Tendermint in `end_block`, despite the value of this field. We need to maintain
    /// this field for non-`Active` validators to trigger state transitions into `Active` when the validator's
    /// potential voting power pushes them into the consensus set.
    pub voting_power: u64,
    /// The validator's current state.
    pub state: State,
}

impl Protobuf<pb::ValidatorStatus> for Status {}

impl From<Status> for pb::ValidatorStatus {
    fn from(v: Status) -> Self {
        pb::ValidatorStatus {
            identity_key: Some(v.identity_key.into()),
            voting_power: v.voting_power,
            state: Some(match v.state {
                State::Inactive => pb::ValidatorState {
                    state: pb::validator_state::ValidatorStateEnum::Inactive as i32,
                    unbonding_epoch: None,
                },
                State::Active => pb::ValidatorState {
                    state: pb::validator_state::ValidatorStateEnum::Active as i32,
                    unbonding_epoch: None,
                },
                State::Unbonding { unbonding_epoch } => pb::ValidatorState {
                    state: pb::validator_state::ValidatorStateEnum::Unbonding as i32,
                    unbonding_epoch: Some(unbonding_epoch),
                },
                State::Slashed => pb::ValidatorState {
                    state: pb::validator_state::ValidatorStateEnum::Slashed as i32,
                    unbonding_epoch: None,
                },
            }),
        }
    }
}

impl TryFrom<pb::ValidatorStatus> for Status {
    type Error = anyhow::Error;
    fn try_from(v: pb::ValidatorStatus) -> Result<Self, Self::Error> {
        let state = match pb::validator_state::ValidatorStateEnum::from_i32(
            v.state.as_ref().unwrap().state,
        )
        .ok_or_else(|| anyhow::anyhow!("missing validator state"))?
        {
            pb::validator_state::ValidatorStateEnum::Inactive => State::Inactive,
            pb::validator_state::ValidatorStateEnum::Active => State::Active,
            pb::validator_state::ValidatorStateEnum::Unbonding => State::Unbonding {
                unbonding_epoch: v
                    .state
                    .unwrap()
                    .unbonding_epoch
                    .ok_or_else(|| anyhow::anyhow!("missing unbonding epoch"))?,
            },
            pb::validator_state::ValidatorStateEnum::Slashed => State::Slashed,
        };

        Ok(Status {
            identity_key: v
                .identity_key
                .ok_or_else(|| anyhow::anyhow!("missing identity key field in proto"))?
                .try_into()?,
            voting_power: v.voting_power,
            state,
        })
    }
}
