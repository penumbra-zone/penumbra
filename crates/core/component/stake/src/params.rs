use penumbra_proto::core::component::stake::v1alpha1 as pb;
use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::StakeParameters", into = "pb::StakeParameters")]
pub struct StakeParameters {
    pub unbonding_epochs: u64,
    /// The number of validators allowed in the consensus set (Active state).
    pub active_validator_limit: u64,
    /// The base reward rate, expressed in basis points of basis points
    pub base_reward_rate: u64,
    /// The penalty for slashing due to misbehavior, expressed in basis points squared (10^-8)
    pub slashing_penalty_misbehavior: u64,
    /// The penalty for slashing due to downtime, expressed in basis points squared (10^-8)
    pub slashing_penalty_downtime: u64,
    /// The number of blocks in the window to check for downtime.
    pub signed_blocks_window_len: u64,
    /// The maximum number of blocks in the window each validator can miss signing without slashing.
    pub missed_blocks_maximum: u64,
}

impl DomainType for StakeParameters {
    type Proto = pb::StakeParameters;
}

impl TryFrom<pb::StakeParameters> for StakeParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::StakeParameters) -> anyhow::Result<Self> {
        Ok(StakeParameters {
            unbonding_epochs: msg.unbonding_epochs,
            active_validator_limit: msg.active_validator_limit,
            slashing_penalty_downtime: msg.slashing_penalty_downtime,
            slashing_penalty_misbehavior: msg.slashing_penalty_misbehavior,
            base_reward_rate: msg.base_reward_rate,
            missed_blocks_maximum: msg.missed_blocks_maximum,
            signed_blocks_window_len: msg.signed_blocks_window_len,
        })
    }
}

impl From<StakeParameters> for pb::StakeParameters {
    fn from(params: StakeParameters) -> Self {
        pb::StakeParameters {
            unbonding_epochs: params.unbonding_epochs,
            active_validator_limit: params.active_validator_limit,
            signed_blocks_window_len: params.signed_blocks_window_len,
            missed_blocks_maximum: params.missed_blocks_maximum,
            slashing_penalty_downtime: params.slashing_penalty_downtime,
            slashing_penalty_misbehavior: params.slashing_penalty_misbehavior,
            base_reward_rate: params.base_reward_rate,
        }
    }
}

// TODO: defaults are implemented here as well as in the
// `pd::main`
impl Default for StakeParameters {
    fn default() -> Self {
        Self {
            unbonding_epochs: 2,
            active_validator_limit: 80,
            // copied from cosmos hub
            signed_blocks_window_len: 10000,
            missed_blocks_maximum: 9500,
            // 1000 basis points = 10%
            slashing_penalty_misbehavior: 1000_0000,
            // 1 basis point = 0.01%
            slashing_penalty_downtime: 1_0000,
            // 3bps -> 11% return over 365 epochs
            base_reward_rate: 3_0000,
        }
    }
}
