use penumbra_chain::params::ChainParameters;
use penumbra_community_pool::params::CommunityPoolParameters;
use penumbra_distributions::DistributionsParameters;
use penumbra_fee::FeeParameters;
use penumbra_governance::params::GovernanceParameters;
use penumbra_ibc::params::IBCParameters;
use penumbra_proto::core::app::v1alpha1 as pb;
use penumbra_proto::view::v1alpha1 as pb_view;
use penumbra_proto::DomainType;
use penumbra_stake::params::StakeParameters;
use serde::{Deserialize, Serialize};

pub mod change;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(try_from = "pb::AppParameters", into = "pb::AppParameters")]
pub struct AppParameters {
    pub chain_params: ChainParameters,
    pub community_pool_params: CommunityPoolParameters,
    pub distributions_params: DistributionsParameters,
    pub fee_params: FeeParameters,
    pub governance_params: GovernanceParameters,
    pub ibc_params: IBCParameters,
    pub stake_params: StakeParameters,
}

impl DomainType for AppParameters {
    type Proto = pb::AppParameters;
}

impl TryFrom<pb::AppParameters> for AppParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::AppParameters) -> anyhow::Result<Self> {
        Ok(AppParameters {
            chain_params: msg
                .chain_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing chain params"))?
                .try_into()?,
            community_pool_params: msg
                .community_pool_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing Community Pool params"))?
                .try_into()?,
            distributions_params: msg
                .distributions_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing distribution params"))?
                .try_into()?,
            fee_params: msg
                .fee_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing fee params"))?
                .try_into()?,
            governance_params: msg
                .governance_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing governance params"))?
                .try_into()?,
            ibc_params: msg
                .ibc_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing ibc params"))?
                .try_into()?,
            stake_params: msg
                .stake_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing stake params"))?
                .try_into()?,
        })
    }
}

impl From<AppParameters> for pb::AppParameters {
    fn from(params: AppParameters) -> Self {
        pb::AppParameters {
            chain_params: Some(params.chain_params.into()),
            community_pool_params: Some(params.community_pool_params.into()),
            distributions_params: Some(params.distributions_params.into()),
            governance_params: Some(params.governance_params.into()),
            ibc_params: Some(params.ibc_params.into()),
            stake_params: Some(params.stake_params.into()),
            fee_params: Some(params.fee_params.into()),
        }
    }
}
impl TryFrom<pb_view::AppParametersResponse> for AppParameters {
    type Error = anyhow::Error;

    fn try_from(response: pb_view::AppParametersResponse) -> Result<Self, Self::Error> {
        response
            .parameters
            .ok_or_else(|| anyhow::anyhow!("empty AppParametersResponse message"))?
            .try_into()
    }
}

impl TryFrom<pb::AppParametersResponse> for AppParameters {
    type Error = anyhow::Error;

    fn try_from(response: pb::AppParametersResponse) -> Result<Self, Self::Error> {
        response
            .app_parameters
            .ok_or_else(|| anyhow::anyhow!("empty AppParametersResponse message"))?
            .try_into()
    }
}
