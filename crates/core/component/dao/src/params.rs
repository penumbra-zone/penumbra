use penumbra_proto::core::component::dao::v1alpha1 as pb;
use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::DaoParameters", into = "pb::DaoParameters")]
pub struct DaoParameters {
    /// Whether DAO spend proposals are enabled.
    pub dao_spend_proposals_enabled: bool,
}

impl DomainType for DaoParameters {
    type Proto = pb::DaoParameters;
}

impl TryFrom<pb::DaoParameters> for DaoParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::DaoParameters) -> anyhow::Result<Self> {
        Ok(DaoParameters {
            dao_spend_proposals_enabled: msg.dao_spend_proposals_enabled,
        })
    }
}

impl From<DaoParameters> for pb::DaoParameters {
    fn from(params: DaoParameters) -> Self {
        pb::DaoParameters {
            dao_spend_proposals_enabled: params.dao_spend_proposals_enabled,
        }
    }
}

impl Default for DaoParameters {
    fn default() -> Self {
        Self {
            dao_spend_proposals_enabled: true,
        }
    }
}
