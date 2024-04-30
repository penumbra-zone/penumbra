use crate::{Delegate, IdentityKey, Undelegate};
use penumbra_proto::core::component::stake::v1 as pb;
use tendermint::abci::{types::Misbehavior, Event, EventAttributeIndexExt};

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

pub fn tombstone_validator(
    current_height: u64,
    identity_key: IdentityKey,
    evidence: &Misbehavior,
) -> pb::EventTombstoneValidator {
    pb::EventTombstoneValidator {
        evidence_height: evidence.height.value(),
        current_height,
        identity_key: Some(identity_key.into()),
        address: evidence.validator.address.to_vec(),
        voting_power: evidence.validator.power.value(),
    }
}
