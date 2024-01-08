use penumbra_proto::core::component::community_pool::v1alpha1 as pb;
use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    try_from = "pb::CommunityPoolParameters",
    into = "pb::CommunityPoolParameters"
)]
pub struct CommunityPoolParameters {
    /// Whether Community Pool spend proposals are enabled.
    pub community_pool_spend_proposals_enabled: bool,
}

impl DomainType for CommunityPoolParameters {
    type Proto = pb::CommunityPoolParameters;
}

impl TryFrom<pb::CommunityPoolParameters> for CommunityPoolParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::CommunityPoolParameters) -> anyhow::Result<Self> {
        Ok(CommunityPoolParameters {
            community_pool_spend_proposals_enabled: msg.community_pool_spend_proposals_enabled,
        })
    }
}

impl From<CommunityPoolParameters> for pb::CommunityPoolParameters {
    fn from(params: CommunityPoolParameters) -> Self {
        pb::CommunityPoolParameters {
            community_pool_spend_proposals_enabled: params.community_pool_spend_proposals_enabled,
        }
    }
}

impl Default for CommunityPoolParameters {
    fn default() -> Self {
        Self {
            community_pool_spend_proposals_enabled: true,
        }
    }
}
