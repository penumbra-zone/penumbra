use crate::{Delegate, Undelegate};

use penumbra_proto::penumbra::core::component::stake::v1 as pb;

pub fn delegate(delegate: &Delegate) -> pb::EventDelegate {
    pb::EventDelegate {
        validator_identity: Some(delegate.validator_identity.into()),
        epoch_index: delegate.epoch_index,
        unbonded_amount: Some(delegate.unbonded_amount.into()),
        delegation_amount: Some(delegate.delegation_amount.into()),
    }
}

pub fn undelegate(undelegate: &Undelegate) -> pb::EventUndelegate {
    pb::EventUndelegate {
        validator_identity: Some(undelegate.validator_identity.into()),
        start_epoch_index: undelegate.start_epoch_index,
        unbonded_amount: Some(undelegate.unbonded_amount.into()),
        delegation_amount: Some(undelegate.delegation_amount.into()),
    }
}
