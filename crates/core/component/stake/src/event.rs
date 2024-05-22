use crate::{
    rate,
    validator::{BondingState, State, Validator},
    Delegate, IdentityKey, Undelegate,
};
use penumbra_num::Amount;
use penumbra_proto::core::component::stake::v1 as pb;
use tendermint::abci::types::Misbehavior;

pub fn validator_state_change(
    identity_key: IdentityKey,
    state: State,
) -> pb::EventValidatorStateChange {
    pb::EventValidatorStateChange {
        identity_key: Some(identity_key.into()),
        state: Some(state.into()),
    }
}

pub fn validator_voting_power_change(
    identity_key: IdentityKey,
    voting_power: Amount,
) -> pb::EventValidatorVotingPowerChange {
    pb::EventValidatorVotingPowerChange {
        identity_key: Some(identity_key.into()),
        voting_power: Some(voting_power.into()),
    }
}

pub fn validator_bonding_state_change(
    identity_key: IdentityKey,
    bonding_state: BondingState,
) -> pb::EventValidatorBondingStateChange {
    pb::EventValidatorBondingStateChange {
        identity_key: Some(identity_key.into()),
        bonding_state: Some(bonding_state.into()),
    }
}

pub fn validator_rate_data_change(
    identity_key: IdentityKey,
    rate_data: rate::RateData,
) -> pb::EventRateDataChange {
    pb::EventRateDataChange {
        identity_key: Some(identity_key.into()),
        rate_data: Some(rate_data.into()),
    }
}

pub fn validator_definition_upload(validator: Validator) -> pb::EventValidatorDefinitionUpload {
    pb::EventValidatorDefinitionUpload {
        validator: Some(validator.into()),
    }
}

pub fn validator_missed_block(identity_key: IdentityKey) -> pb::EventValidatorMissedBlock {
    pb::EventValidatorMissedBlock {
        identity_key: Some(identity_key.into()),
    }
}

pub fn delegate(delegate: &Delegate) -> pb::EventDelegate {
    pb::EventDelegate {
        validator_identity: Some(delegate.validator_identity.into()),
        amount: Some(delegate.unbonded_amount.into()),
    }
}

pub fn undelegate(undelegate: &Undelegate) -> pb::EventUndelegate {
    pb::EventUndelegate {
        validator_identity: Some(undelegate.validator_identity.into()),
        amount: Some(undelegate.unbonded_amount.into()),
    }
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
