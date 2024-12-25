use crate::Fee;
use anyhow::{anyhow, Context};
use penumbra_sdk_proto::{core::component::fee::v1 as pb, DomainType, Name as _};

#[derive(Clone, Debug)]
pub struct EventBlockFees {
    pub swapped_fee_total: Fee,
    pub swapped_base_fee_total: Fee,
    pub swapped_tip_total: Fee,
}

impl TryFrom<pb::EventBlockFees> for EventBlockFees {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventBlockFees) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventBlockFees) -> anyhow::Result<EventBlockFees> {
            Ok(EventBlockFees {
                swapped_fee_total: value
                    .swapped_fee_total
                    .ok_or(anyhow!("missing `swapped_fee_total`"))?
                    .try_into()?,
                swapped_base_fee_total: value
                    .swapped_base_fee_total
                    .ok_or(anyhow!("missing `swapped_base_fee_total`"))?
                    .try_into()?,
                swapped_tip_total: value
                    .swapped_tip_total
                    .ok_or(anyhow!("missing `swapped_tip_total`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventBlockFees::NAME))
    }
}

impl From<EventBlockFees> for pb::EventBlockFees {
    fn from(value: EventBlockFees) -> Self {
        Self {
            swapped_fee_total: Some(value.swapped_fee_total.into()),
            swapped_base_fee_total: Some(value.swapped_base_fee_total.into()),
            swapped_tip_total: Some(value.swapped_tip_total.into()),
        }
    }
}

impl DomainType for EventBlockFees {
    type Proto = pb::EventBlockFees;
}
