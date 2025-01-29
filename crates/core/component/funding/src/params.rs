use penumbra_sdk_num::Percentage;
use penumbra_sdk_proto::core::component::funding::v1 as pb;
use penumbra_sdk_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiquidityTournamentParameters {
    // The fraction of gauge votes that an asset must clear to receive rewards.
    pub gauge_threshold: Percentage,
    // The maximum number of liquidity positions that can receive rewards.
    pub max_positions: u64,
    // The maximum number of delegators that can receive rewards.
    pub max_delegators: u64,
    // The share of rewards that go to delegators, instead of positions.
    pub delegator_share: Percentage,
}

impl Default for LiquidityTournamentParameters {
    fn default() -> Self {
        Self {
            gauge_threshold: Percentage::from_percent(100),
            max_positions: 0,
            max_delegators: 0,
            delegator_share: Percentage::zero(),
        }
    }
}

impl TryFrom<pb::funding_parameters::LiquidityTournament> for LiquidityTournamentParameters {
    type Error = anyhow::Error;

    fn try_from(proto: pb::funding_parameters::LiquidityTournament) -> Result<Self, Self::Error> {
        Ok(Self {
            gauge_threshold: Percentage::from_percent(proto.gauge_threshold_percent),
            max_positions: proto.max_positions,
            max_delegators: proto.max_delegators,
            delegator_share: Percentage::from_percent(proto.delegator_share_percent),
        })
    }
}

impl From<LiquidityTournamentParameters> for pb::funding_parameters::LiquidityTournament {
    fn from(value: LiquidityTournamentParameters) -> Self {
        Self {
            gauge_threshold_percent: value.gauge_threshold.to_percent(),
            max_positions: value.max_positions,
            max_delegators: value.max_delegators,
            delegator_share_percent: value.delegator_share.to_percent(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::FundingParameters", into = "pb::FundingParameters")]
pub struct FundingParameters {
    pub liquidity_tournament: LiquidityTournamentParameters,
}

impl DomainType for FundingParameters {
    type Proto = pb::FundingParameters;
}

impl TryFrom<pb::FundingParameters> for FundingParameters {
    type Error = anyhow::Error;

    fn try_from(proto: pb::FundingParameters) -> anyhow::Result<Self> {
        Ok(FundingParameters {
            // Explicitly consider missing parameters to *be* the default parameters, for upgrades.
            liquidity_tournament: proto
                .liquidity_tournament
                .map(LiquidityTournamentParameters::try_from)
                .transpose()?
                .unwrap_or_default(),
        })
    }
}

impl From<FundingParameters> for pb::FundingParameters {
    fn from(params: FundingParameters) -> Self {
        pb::FundingParameters {
            liquidity_tournament: Some(params.liquidity_tournament.into()),
        }
    }
}

impl Default for FundingParameters {
    fn default() -> Self {
        Self {
            liquidity_tournament: LiquidityTournamentParameters::default(),
        }
    }
}
