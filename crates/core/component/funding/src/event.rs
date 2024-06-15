use penumbra_num::Amount;
use penumbra_proto::penumbra::core::component::funding::v1 as pb;

pub fn funding_stream_reward(
    recipient: String,
    epoch_index: u64,
    reward_amount: Amount,
) -> pb::EventFundingStreamReward {
    pb::EventFundingStreamReward {
        recipient,
        epoch_index,
        reward_amount: Some(reward_amount.into()),
    }
}
