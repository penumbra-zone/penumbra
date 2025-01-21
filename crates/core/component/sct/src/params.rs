use penumbra_sdk_proto::penumbra::core::component::sct::v1 as pb;
use penumbra_sdk_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::SctParameters", into = "pb::SctParameters")]
/// The configuration parameters for the SCT component.
pub struct SctParameters {
    /// The "default" duration of an epoch in number of blocks.
    /// Note that this is a soft target, and a variety of events
    /// can trigger an epoch transition.
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
            // Measured in blocks, assuming a 5s block time
            // this is about a day worth of blocks.
            epoch_duration: 17280,
        }
    }
}
