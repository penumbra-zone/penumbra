use penumbra_proto::penumbra::core::component::sct::v1alpha1 as pb;
use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::SctParameters", into = "pb::SctParameters")]
pub struct SctParameters {
    pub epoch_duration: u64,
}

impl DomainType for SctParameters {
    type Proto = pb::SctParameters;
}

impl TryFrom<pb::SctParameters> for SctParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::SctParameters) -> anyhow::Result<Self> {
        Ok(SctParameters {
            epoch_duration: msg.epoch_duration,
        })
    }
}

impl From<SctParameters> for pb::SctParameters {
    fn from(params: SctParameters) -> Self {
        pb::SctParameters {
            epoch_duration: params.epoch_duration,
        }
    }
}

impl Default for SctParameters {
    fn default() -> Self {
        Self {
            chain_id: String::new(),
            epoch_duration: 719,
        }
    }
}
