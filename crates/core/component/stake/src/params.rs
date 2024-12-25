use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::core::component::stake::v1 as pb;
use penumbra_sdk_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::StakeParameters", into = "pb::StakeParameters")]
pub struct StakeParameters {
    /// The number of blocks to wait before a validator can unbond their stake.
    pub unbonding_delay: u64,
    /// The number of validators allowed in the consensus set (Active state).
    pub active_validator_limit: u64,
    /// The penalty for slashing due to misbehavior, expressed in basis points squared (10^-8)
    pub slashing_penalty_misbehavior: u64,
    /// The penalty for slashing due to downtime, expressed in basis points squared (10^-8)
    pub slashing_penalty_downtime: u64,
    /// The number of blocks in the window to check for downtime.
    pub signed_blocks_window_len: u64,
    /// The maximum number of blocks in the window each validator can miss signing without slashing.
    pub missed_blocks_maximum: u64,
    /// The minimum amount of stake required for a validator to be indexed.
    pub min_validator_stake: Amount,
}

impl DomainType for StakeParameters {
    type Proto = pb::StakeParameters;
}

impl TryFrom<pb::StakeParameters> for StakeParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::StakeParameters) -> anyhow::Result<Self> {
        Ok(StakeParameters {
            active_validator_limit: msg.active_validator_limit,
            slashing_penalty_downtime: msg.slashing_penalty_downtime,
            slashing_penalty_misbehavior: msg.slashing_penalty_misbehavior,
            missed_blocks_maximum: msg.missed_blocks_maximum,
            signed_blocks_window_len: msg.signed_blocks_window_len,
            min_validator_stake: msg
                .min_validator_stake
                .ok_or_else(|| anyhow::anyhow!("missing min_validator_stake"))?
                .try_into()?,
            unbonding_delay: msg.unbonding_delay,
        })
    }
}

impl From<StakeParameters> for pb::StakeParameters {
    #[allow(deprecated)]
    fn from(params: StakeParameters) -> Self {
        pb::StakeParameters {
            unbonding_epochs: 0,
            active_validator_limit: params.active_validator_limit,
            signed_blocks_window_len: params.signed_blocks_window_len,
            missed_blocks_maximum: params.missed_blocks_maximum,
            slashing_penalty_downtime: params.slashing_penalty_downtime,
            slashing_penalty_misbehavior: params.slashing_penalty_misbehavior,
            base_reward_rate: 0,
            min_validator_stake: Some(params.min_validator_stake.into()),
            unbonding_delay: params.unbonding_delay,
        }
    }
}

impl Default for StakeParameters {
    fn default() -> Self {
        Self {
            // About a week worth of blocks.
            unbonding_delay: 120960,
            active_validator_limit: 80,
            // Copied from cosmos hub
            signed_blocks_window_len: 10000,
            missed_blocks_maximum: 9500,
            // 1000 basis points = 10%
            slashing_penalty_misbehavior: 1000_0000,
            // 1 basis point = 0.01%
            slashing_penalty_downtime: 1_0000,
            // 1 penumbra
            min_validator_stake: 1_000_000u128.into(),
        }
    }
}
