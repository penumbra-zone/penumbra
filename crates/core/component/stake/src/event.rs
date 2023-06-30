use crate::{Delegate, Undelegate};
use tendermint::abci::{Event, EventAttributeIndexExt};

pub fn delegate(delegate: &Delegate) -> Event {
    Event::new(
        "action_delegate",
        [
            ("validator", delegate.validator_identity.to_string()).index(),
            ("amount", delegate.unbonded_amount.to_string()).no_index(),
        ],
    )
}

pub fn undelegate(undelegate: &Undelegate) -> Event {
    Event::new(
        "action_undelegate",
        [
            ("validator", undelegate.validator_identity.to_string()).index(),
            ("amount", undelegate.unbonded_amount.to_string()).no_index(),
        ],
    )
}
