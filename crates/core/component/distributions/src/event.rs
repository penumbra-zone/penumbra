use anyhow::{anyhow, Context as _};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{
    penumbra::core::component::distributions::v1 as pb, DomainType, Name as _,
};

#[derive(Clone, Debug)]
pub struct EventLqtPoolSizeIncrease {
    pub epoch_index: u64,
    pub increase: Amount,
    pub new_total: Amount,
}

impl TryFrom<pb::EventLqtPoolSizeIncrease> for EventLqtPoolSizeIncrease {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventLqtPoolSizeIncrease) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventLqtPoolSizeIncrease) -> anyhow::Result<EventLqtPoolSizeIncrease> {
            Ok(EventLqtPoolSizeIncrease {
                epoch_index: value.epoch,
                increase: value
                    .increase
                    .ok_or(anyhow!("missing `increase`"))?
                    .try_into()?,
                new_total: value
                    .new_total
                    .ok_or(anyhow!("missing `new_total`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventLqtPoolSizeIncrease::NAME))
    }
}

impl From<EventLqtPoolSizeIncrease> for pb::EventLqtPoolSizeIncrease {
    fn from(value: EventLqtPoolSizeIncrease) -> Self {
        Self {
            epoch: value.epoch_index,
            increase: Some(value.increase.into()),
            new_total: Some(value.new_total.into()),
        }
    }
}

impl DomainType for EventLqtPoolSizeIncrease {
    type Proto = pb::EventLqtPoolSizeIncrease;
}
