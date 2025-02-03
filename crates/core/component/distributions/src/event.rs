use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::penumbra::core::component::distributions::v1 as pb;

/// Event for when LQT pool size increases.
pub fn event_lqt_pool_size_increase(
    epoch: u64,
    increase: Amount,
    new_total: Amount,
) -> pb::EventLqtPoolSizeIncrease {
    pb::EventLqtPoolSizeIncrease {
        epoch,
        increase: Some(increase.into()),
        new_total: Some(new_total.into()),
    }
}
