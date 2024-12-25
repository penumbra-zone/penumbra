use anyhow::{anyhow, Context};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{penumbra::core::component::funding::v1 as pb, DomainType, Name as _};

#[derive(Clone, Debug)]
pub struct EventFundingStreamReward {
    pub recipient: String,
    pub epoch_index: u64,
    pub reward_amount: Amount,
}

impl TryFrom<pb::EventFundingStreamReward> for EventFundingStreamReward {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventFundingStreamReward) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventFundingStreamReward) -> anyhow::Result<EventFundingStreamReward> {
            Ok(EventFundingStreamReward {
                recipient: value.recipient,
                epoch_index: value.epoch_index,
                reward_amount: value
                    .reward_amount
                    .ok_or(anyhow!("missing `reward_amount`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventFundingStreamReward::NAME))
    }
}

impl From<EventFundingStreamReward> for pb::EventFundingStreamReward {
    fn from(value: EventFundingStreamReward) -> Self {
        Self {
            recipient: value.recipient,
            epoch_index: value.epoch_index,
            reward_amount: Some(value.reward_amount.into()),
        }
    }
}

impl DomainType for EventFundingStreamReward {
    type Proto = pb::EventFundingStreamReward;
}
