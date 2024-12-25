use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{penumbra::core::component::stake::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::{validator::BondingState, validator::State, IdentityKey};

/// The current status of a validator, including its identity, voting power, and state in the
/// validator state machine.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorStatus", into = "pb::ValidatorStatus")]
pub struct Status {
    /// The validator's identity.
    pub identity_key: IdentityKey,
    /// The validator's voting power.
    pub voting_power: Amount,
    /// The validator's current state.
    pub state: State,
    /// Represents the bonding status of the validator's stake pool.
    pub bonding_state: BondingState,
}

impl DomainType for Status {
    type Proto = pb::ValidatorStatus;
}

impl From<Status> for pb::ValidatorStatus {
    fn from(v: Status) -> Self {
        pb::ValidatorStatus {
            identity_key: Some(v.identity_key.into()),
            voting_power: Some(v.voting_power.into()),
            bonding_state: Some(v.bonding_state.into()),
            state: Some(v.state.into()),
        }
    }
}

impl TryFrom<pb::ValidatorStatus> for Status {
    type Error = anyhow::Error;
    fn try_from(v: pb::ValidatorStatus) -> Result<Self, Self::Error> {
        Ok(Status {
            identity_key: v
                .identity_key
                .ok_or_else(|| anyhow::anyhow!("missing identity key field in proto"))?
                .try_into()?,
            voting_power: v
                .voting_power
                .ok_or_else(|| anyhow::anyhow!("missing voting power field in proto"))?
                .try_into()?,
            state: v
                .state
                .ok_or_else(|| anyhow::anyhow!("missing state field in proto"))?
                .try_into()?,
            bonding_state: v
                .bonding_state
                .expect("expected bonding state to be set on validator status")
                .try_into()?,
        })
    }
}
