use penumbra_sdk_auction::params::AuctionParameters;
use penumbra_sdk_community_pool::params::CommunityPoolParameters;
use penumbra_sdk_dex::DexParameters;
use penumbra_sdk_distributions::DistributionsParameters;
use penumbra_sdk_fee::FeeParameters;
use penumbra_sdk_funding::FundingParameters;
use penumbra_sdk_governance::params::GovernanceParameters;
use penumbra_sdk_ibc::params::IBCParameters;
use penumbra_sdk_proto::core::app::v1 as pb;
use penumbra_sdk_proto::view::v1 as pb_view;
use penumbra_sdk_proto::DomainType;
use penumbra_sdk_sct::params::SctParameters;
use penumbra_sdk_shielded_pool::params::ShieldedPoolParameters;
use penumbra_sdk_stake::params::StakeParameters;
use serde::{Deserialize, Serialize};

pub mod change;

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(try_from = "pb::AppParameters", into = "pb::AppParameters")]
pub struct AppParameters {
    pub chain_id: String,
    pub auction_params: AuctionParameters,
    pub community_pool_params: CommunityPoolParameters,
    pub distributions_params: DistributionsParameters,
    pub dex_params: DexParameters,
    pub fee_params: FeeParameters,
    pub funding_params: FundingParameters,
    pub governance_params: GovernanceParameters,
    pub ibc_params: IBCParameters,
    pub sct_params: SctParameters,
    pub shielded_pool_params: ShieldedPoolParameters,
    pub stake_params: StakeParameters,
}

impl DomainType for AppParameters {
    type Proto = pb::AppParameters;
}

impl TryFrom<pb::AppParameters> for AppParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::AppParameters) -> anyhow::Result<Self> {
        Ok(AppParameters {
            chain_id: msg.chain_id,
            auction_params: msg
                .auction_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing auction params"))?
                .try_into()?,
            community_pool_params: msg
                .community_pool_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing community pool params"))?
                .try_into()?,
            distributions_params: msg
                .distributions_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing distribution params"))?
                .try_into()?,
            fee_params: msg
                .fee_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing fee params"))?
                .try_into()?,
            funding_params: msg
                .funding_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing funding params"))?
                .try_into()?,
            governance_params: msg
                .governance_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing governance params"))?
                .try_into()?,
            ibc_params: msg
                .ibc_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing ibc params"))?
                .try_into()?,
            sct_params: msg
                .sct_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing sct params"))?
                .try_into()?,
            shielded_pool_params: msg
                .shielded_pool_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing shielded pool params"))?
                .try_into()?,
            stake_params: msg
                .stake_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing stake params"))?
                .try_into()?,
            dex_params: msg
                .dex_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing dex params"))?
                .try_into()?,
        })
    }
}

impl From<AppParameters> for pb::AppParameters {
    fn from(params: AppParameters) -> Self {
        pb::AppParameters {
            chain_id: params.chain_id,
            auction_params: Some(params.auction_params.into()),
            community_pool_params: Some(params.community_pool_params.into()),
            distributions_params: Some(params.distributions_params.into()),
            fee_params: Some(params.fee_params.into()),
            funding_params: Some(params.funding_params.into()),
            governance_params: Some(params.governance_params.into()),
            ibc_params: Some(params.ibc_params.into()),
            sct_params: Some(params.sct_params.into()),
            shielded_pool_params: Some(params.shielded_pool_params.into()),
            stake_params: Some(params.stake_params.into()),
            dex_params: Some(params.dex_params.into()),
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
