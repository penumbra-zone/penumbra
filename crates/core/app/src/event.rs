use anyhow::{anyhow, Context};
use penumbra_sdk_proto::{penumbra::core::app::v1 as pb, DomainType};
use prost::Name as _;

use crate::params::AppParameters;

#[derive(Clone, Debug)]
pub struct EventAppParametersChange {
    pub new_parameters: AppParameters,
}

impl TryFrom<pb::EventAppParametersChange> for EventAppParametersChange {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventAppParametersChange) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventAppParametersChange) -> anyhow::Result<EventAppParametersChange> {
            Ok(EventAppParametersChange {
                new_parameters: value
                    .new_parameters
                    .ok_or(anyhow!("missing `new_parameters`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventAppParametersChange::NAME))
    }
}

impl From<EventAppParametersChange> for pb::EventAppParametersChange {
    fn from(value: EventAppParametersChange) -> Self {
        Self {
            new_parameters: Some(value.new_parameters.into()),
        }
    }
}

impl DomainType for EventAppParametersChange {
    type Proto = pb::EventAppParametersChange;
}
