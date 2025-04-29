use penumbra_sdk_proto::core::component::distributions::v1 as pb;
use penumbra_sdk_proto::DomainType;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    try_from = "pb::DistributionsParameters",
    into = "pb::DistributionsParameters"
)]
pub struct DistributionsParameters {
    pub staking_issuance_per_block: u64,
    pub liquidity_tournament_incentive_per_block: u64,
    pub liquidity_tournament_end_block: Option<NonZeroU64>,
}

impl DomainType for DistributionsParameters {
    type Proto = pb::DistributionsParameters;
}

impl TryFrom<pb::DistributionsParameters> for DistributionsParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::DistributionsParameters) -> anyhow::Result<Self> {
        Ok(DistributionsParameters {
            staking_issuance_per_block: msg.staking_issuance_per_block,
            liquidity_tournament_incentive_per_block: msg.liquidity_tournament_incentive_per_block,
            liquidity_tournament_end_block: NonZeroU64::new(msg.liquidity_tournament_end_block),
        })
    }
}

impl From<DistributionsParameters> for pb::DistributionsParameters {
    fn from(params: DistributionsParameters) -> Self {
        pb::DistributionsParameters {
            staking_issuance_per_block: params.staking_issuance_per_block,
            liquidity_tournament_incentive_per_block: params
                .liquidity_tournament_incentive_per_block,
            liquidity_tournament_end_block: params
                .liquidity_tournament_end_block
                .map_or(0, NonZeroU64::get),
        }
    }
}

impl Default for DistributionsParameters {
    fn default() -> Self {
        Self {
            staking_issuance_per_block: 1_000_000,
            liquidity_tournament_incentive_per_block: 0,
            liquidity_tournament_end_block: None,
        }
    }
}
