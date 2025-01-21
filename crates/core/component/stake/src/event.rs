use crate::{
    rate::RateData,
    validator::{BondingState, State, Validator},
    Delegate, IdentityKey, Penalty, Undelegate,
};
use anyhow::{anyhow, Context as _};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{core::component::stake::v1 as pb, DomainType, Name as _};
use tendermint::abci::types::Misbehavior;

#[derive(Clone, Debug)]
pub struct EventValidatorStateChange {
    pub identity_key: IdentityKey,
    pub state: State,
}

impl TryFrom<pb::EventValidatorStateChange> for EventValidatorStateChange {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventValidatorStateChange) -> Result<Self, Self::Error> {
        fn inner(
            value: pb::EventValidatorStateChange,
        ) -> anyhow::Result<EventValidatorStateChange> {
            Ok(EventValidatorStateChange {
                identity_key: value
                    .identity_key
                    .ok_or(anyhow!("missing `identity_key`"))?
                    .try_into()?,
                state: value.state.ok_or(anyhow!("missing `state`"))?.try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventValidatorStateChange::NAME))
    }
}

impl From<EventValidatorStateChange> for pb::EventValidatorStateChange {
    fn from(value: EventValidatorStateChange) -> Self {
        Self {
            identity_key: Some(value.identity_key.into()),
            state: Some(value.state.into()),
        }
    }
}

impl DomainType for EventValidatorStateChange {
    type Proto = pb::EventValidatorStateChange;
}

#[derive(Clone, Debug)]
pub struct EventValidatorVotingPowerChange {
    pub identity_key: IdentityKey,
    pub voting_power: Amount,
}

impl TryFrom<pb::EventValidatorVotingPowerChange> for EventValidatorVotingPowerChange {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventValidatorVotingPowerChange) -> Result<Self, Self::Error> {
        fn inner(
            value: pb::EventValidatorVotingPowerChange,
        ) -> anyhow::Result<EventValidatorVotingPowerChange> {
            Ok(EventValidatorVotingPowerChange {
                identity_key: value
                    .identity_key
                    .ok_or(anyhow!("missing `identity_key`"))?
                    .try_into()?,
                voting_power: value
                    .voting_power
                    .ok_or(anyhow!("missing `voting_power`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!(
            "parsing {}",
            pb::EventValidatorVotingPowerChange::NAME
        ))
    }
}

impl From<EventValidatorVotingPowerChange> for pb::EventValidatorVotingPowerChange {
    fn from(value: EventValidatorVotingPowerChange) -> Self {
        Self {
            identity_key: Some(value.identity_key.into()),
            voting_power: Some(value.voting_power.into()),
        }
    }
}

impl DomainType for EventValidatorVotingPowerChange {
    type Proto = pb::EventValidatorVotingPowerChange;
}

#[derive(Clone, Debug)]
pub struct EventValidatorBondingStateChange {
    pub identity_key: IdentityKey,
    pub bonding_state: BondingState,
}

impl TryFrom<pb::EventValidatorBondingStateChange> for EventValidatorBondingStateChange {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventValidatorBondingStateChange) -> Result<Self, Self::Error> {
        fn inner(
            value: pb::EventValidatorBondingStateChange,
        ) -> anyhow::Result<EventValidatorBondingStateChange> {
            Ok(EventValidatorBondingStateChange {
                identity_key: value
                    .identity_key
                    .ok_or(anyhow!("missing `identity_key`"))?
                    .try_into()?,
                bonding_state: value
                    .bonding_state
                    .ok_or(anyhow!("missing `bonding_state`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!(
            "parsing {}",
            pb::EventValidatorBondingStateChange::NAME
        ))
    }
}

impl From<EventValidatorBondingStateChange> for pb::EventValidatorBondingStateChange {
    fn from(value: EventValidatorBondingStateChange) -> Self {
        Self {
            identity_key: Some(value.identity_key.into()),
            bonding_state: Some(value.bonding_state.into()),
        }
    }
}

impl DomainType for EventValidatorBondingStateChange {
    type Proto = pb::EventValidatorBondingStateChange;
}

#[derive(Clone, Debug)]
pub struct EventRateDataChange {
    pub identity_key: IdentityKey,
    pub rate_data: RateData,
}

impl TryFrom<pb::EventRateDataChange> for EventRateDataChange {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventRateDataChange) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventRateDataChange) -> anyhow::Result<EventRateDataChange> {
            Ok(EventRateDataChange {
                identity_key: value
                    .identity_key
                    .ok_or(anyhow!("missing `identity_key`"))?
                    .try_into()?,
                rate_data: value
                    .rate_data
                    .ok_or(anyhow!("missing `rate_data`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventRateDataChange::NAME))
    }
}

impl From<EventRateDataChange> for pb::EventRateDataChange {
    fn from(value: EventRateDataChange) -> Self {
        Self {
            identity_key: Some(value.identity_key.into()),
            rate_data: Some(value.rate_data.into()),
        }
    }
}

impl DomainType for EventRateDataChange {
    type Proto = pb::EventRateDataChange;
}

#[derive(Clone, Debug)]
pub struct EventValidatorDefinitionUpload {
    pub validator: Validator,
}

impl TryFrom<pb::EventValidatorDefinitionUpload> for EventValidatorDefinitionUpload {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventValidatorDefinitionUpload) -> Result<Self, Self::Error> {
        fn inner(
            value: pb::EventValidatorDefinitionUpload,
        ) -> anyhow::Result<EventValidatorDefinitionUpload> {
            Ok(EventValidatorDefinitionUpload {
                validator: value
                    .validator
                    .ok_or(anyhow!("missing `validator`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!(
            "parsing {}",
            pb::EventValidatorDefinitionUpload::NAME
        ))
    }
}

impl From<EventValidatorDefinitionUpload> for pb::EventValidatorDefinitionUpload {
    fn from(value: EventValidatorDefinitionUpload) -> Self {
        Self {
            validator: Some(value.validator.into()),
        }
    }
}

impl DomainType for EventValidatorDefinitionUpload {
    type Proto = pb::EventValidatorDefinitionUpload;
}

#[derive(Clone, Debug)]
pub struct EventValidatorMissedBlock {
    pub identity_key: IdentityKey,
}

impl TryFrom<pb::EventValidatorMissedBlock> for EventValidatorMissedBlock {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventValidatorMissedBlock) -> Result<Self, Self::Error> {
        fn inner(
            value: pb::EventValidatorMissedBlock,
        ) -> anyhow::Result<EventValidatorMissedBlock> {
            Ok(EventValidatorMissedBlock {
                identity_key: value
                    .identity_key
                    .ok_or(anyhow!("missing `identity_key`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventValidatorMissedBlock::NAME))
    }
}

impl From<EventValidatorMissedBlock> for pb::EventValidatorMissedBlock {
    fn from(value: EventValidatorMissedBlock) -> Self {
        Self {
            identity_key: Some(value.identity_key.into()),
        }
    }
}

impl DomainType for EventValidatorMissedBlock {
    type Proto = pb::EventValidatorMissedBlock;
}

#[derive(Clone, Debug)]
pub struct EventDelegate {
    pub identity_key: IdentityKey,
    pub amount: Amount,
}

impl From<&Delegate> for EventDelegate {
    fn from(value: &Delegate) -> Self {
        Self {
            identity_key: value.validator_identity,
            amount: value.unbonded_amount,
        }
    }
}

impl TryFrom<pb::EventDelegate> for EventDelegate {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventDelegate) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventDelegate) -> anyhow::Result<EventDelegate> {
            Ok(EventDelegate {
                identity_key: value
                    .identity_key
                    .ok_or(anyhow!("missing `identity_key`"))?
                    .try_into()?,
                amount: value
                    .amount
                    .ok_or(anyhow!("missing `amount`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventDelegate::NAME))
    }
}

impl From<EventDelegate> for pb::EventDelegate {
    fn from(value: EventDelegate) -> Self {
        Self {
            identity_key: Some(value.identity_key.into()),
            amount: Some(value.amount.into()),
        }
    }
}

impl DomainType for EventDelegate {
    type Proto = pb::EventDelegate;
}

#[derive(Clone, Debug)]
pub struct EventUndelegate {
    pub identity_key: IdentityKey,
    pub amount: Amount,
}

impl From<&Undelegate> for EventUndelegate {
    fn from(value: &Undelegate) -> Self {
        Self {
            identity_key: value.validator_identity,
            amount: value.unbonded_amount,
        }
    }
}

impl TryFrom<pb::EventUndelegate> for EventUndelegate {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventUndelegate) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventUndelegate) -> anyhow::Result<EventUndelegate> {
            Ok(EventUndelegate {
                identity_key: value
                    .identity_key
                    .ok_or(anyhow!("missing `identity_key`"))?
                    .try_into()?,
                amount: value
                    .amount
                    .ok_or(anyhow!("missing `amount`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventUndelegate::NAME))
    }
}

impl From<EventUndelegate> for pb::EventUndelegate {
    fn from(value: EventUndelegate) -> Self {
        Self {
            identity_key: Some(value.identity_key.into()),
            amount: Some(value.amount.into()),
        }
    }
}

impl DomainType for EventUndelegate {
    type Proto = pb::EventUndelegate;
}

#[derive(Clone, Debug)]
pub struct EventTombstoneValidator {
    pub evidence_height: u64,
    pub current_height: u64,
    pub identity_key: IdentityKey,
    pub address: Vec<u8>,
    pub voting_power: u64,
}

impl EventTombstoneValidator {
    pub fn from_evidence(
        current_height: u64,
        identity_key: IdentityKey,
        evidence: &Misbehavior,
    ) -> Self {
        Self {
            evidence_height: evidence.height.value(),
            current_height,
            identity_key,
            address: evidence.validator.address.to_vec(),
            voting_power: evidence.validator.power.value(),
        }
    }
}

impl TryFrom<pb::EventTombstoneValidator> for EventTombstoneValidator {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventTombstoneValidator) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventTombstoneValidator) -> anyhow::Result<EventTombstoneValidator> {
            Ok(EventTombstoneValidator {
                evidence_height: value.evidence_height,
                current_height: value.current_height,
                identity_key: value
                    .identity_key
                    .ok_or(anyhow!("missing `identity_key`"))?
                    .try_into()?,
                address: value.address,
                voting_power: value.voting_power,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventTombstoneValidator::NAME))
    }
}

impl From<EventTombstoneValidator> for pb::EventTombstoneValidator {
    fn from(value: EventTombstoneValidator) -> Self {
        Self {
            evidence_height: value.evidence_height,
            current_height: value.current_height,
            identity_key: Some(value.identity_key.into()),
            address: value.address,
            voting_power: value.voting_power,
        }
    }
}

impl DomainType for EventTombstoneValidator {
    type Proto = pb::EventTombstoneValidator;
}

#[derive(Clone, Debug)]
pub struct EventSlashingPenaltyApplied {
    pub identity_key: IdentityKey,
    pub epoch_index: u64,
    pub new_penalty: Penalty,
}

impl TryFrom<pb::EventSlashingPenaltyApplied> for EventSlashingPenaltyApplied {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventSlashingPenaltyApplied) -> Result<Self, Self::Error> {
        fn inner(
            value: pb::EventSlashingPenaltyApplied,
        ) -> anyhow::Result<EventSlashingPenaltyApplied> {
            Ok(EventSlashingPenaltyApplied {
                identity_key: value
                    .identity_key
                    .ok_or(anyhow!("missing `identity_key`"))?
                    .try_into()?,
                epoch_index: value.epoch_index,
                new_penalty: value
                    .new_penalty
                    .ok_or(anyhow!("missing `new_penalty`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventSlashingPenaltyApplied::NAME))
    }
}

impl From<EventSlashingPenaltyApplied> for pb::EventSlashingPenaltyApplied {
    fn from(value: EventSlashingPenaltyApplied) -> Self {
        Self {
            identity_key: Some(value.identity_key.into()),
            epoch_index: value.epoch_index,
            new_penalty: Some(value.new_penalty.into()),
        }
    }
}

impl DomainType for EventSlashingPenaltyApplied {
    type Proto = pb::EventSlashingPenaltyApplied;
}
