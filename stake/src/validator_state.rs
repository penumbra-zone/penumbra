use std::{collections::BTreeMap, str::FromStr};

use anyhow::Result;

use tendermint::PublicKey;

use crate::{IdentityKey, ValidatorInfo};

#[derive(Debug, Clone)]
pub struct ValidatorStateMachine {
    /// Records validator states as they change during the course of the block.
    ///
    /// Updated as changes occur during the block, but will not be persisted
    /// to the database until the block is committed.
    validator_states: BTreeMap<IdentityKey, ValidatorState>,
    /// List of validators that exist during the lifespan of the block.
    block_validators: Vec<ValidatorInfo>,
}

impl ValidatorStateMachine {
    pub fn new(block_validators: Vec<ValidatorInfo>) -> Self {
        // Initialize all state machine validator states to their current state from the block validators.
        let mut validator_states = BTreeMap::new();
        for validator in block_validators.iter() {
            validator_states.insert(
                validator.validator.identity_key.clone(),
                validator.status.state.clone(),
            );
        }

        ValidatorStateMachine {
            validator_states,
            block_validators,
        }
    }

    pub fn get_state(&self, identity_key: &IdentityKey) -> Option<&ValidatorState> {
        self.validator_states.get(identity_key)
    }

    pub fn transition(
        &mut self,
        identity_key: &IdentityKey,
        event: ValidatorStateEvent,
    ) -> Result<()> {
        // Enforce the semantics of the state machine by using the current state and the
        // data contained within the event to determine the next state.
        let current_state = self
            .get_state(identity_key)
            .ok_or(anyhow::anyhow!("validator must exist to transition state"))?;
        match event {
            ValidatorStateEvent::Activate => match current_state {
                ValidatorState::Inactive => {
                    self.validator_states
                        .insert(identity_key.clone(), ValidatorState::Active);
                    Ok(())
                }
                ValidatorState::Unbonding { unbonding_epoch: _ } => {
                    self.validator_states
                        .insert(identity_key.clone(), ValidatorState::Active);
                    Ok(())
                }
                _ => Err(anyhow::anyhow!(
                    "only inactive or unbonding validators can move to active state"
                )),
            },
            ValidatorStateEvent::Deactivate => match current_state {
                ValidatorState::Unbonding { unbonding_epoch: _ } => {
                    self.validator_states
                        .insert(identity_key.clone(), ValidatorState::Inactive);
                    Ok(())
                }
                _ => Err(anyhow::anyhow!(
                    "only unbonding validators can move to inactive state"
                )),
            },
            ValidatorStateEvent::Unbond(unbonding_epoch) => match current_state {
                ValidatorState::Active => {
                    self.validator_states.insert(
                        identity_key.clone(),
                        ValidatorState::Unbonding { unbonding_epoch },
                    );
                    Ok(())
                }
                _ => Err(anyhow::anyhow!(
                    "only active validators can move to unbonding state"
                )),
            },
            ValidatorStateEvent::Slash => match current_state {
                ValidatorState::Active => {
                    self.validator_states
                        .insert(identity_key.clone(), ValidatorState::Slashed);
                    Ok(())
                }
                ValidatorState::Unbonding { unbonding_epoch: _ } => {
                    self.validator_states
                        .insert(identity_key.clone(), ValidatorState::Slashed);
                    Ok(())
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "only active or unbonding validators can move to slashed state"
                    ))
                }
            },
        }
    }

    pub fn transition_validator(
        &mut self,
        ck: PublicKey,
        event: ValidatorStateEvent,
    ) -> Result<()> {
        let validator_info = self
            .block_validators
            .iter()
            .find(|v| v.validator.consensus_key == ck)
            .cloned()
            .ok_or(anyhow::anyhow!("No validator found"))?;

        self.transition(&validator_info.validator.identity_key, event)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// An event triggering a validator state transition.
pub enum ValidatorStateEvent {
    Slash,
    Unbond(u64),
    Activate,
    Deactivate,
}

/// The state of a validator in the validator state machine.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValidatorState {
    /// The validator is not currently a part of the consensus set, but could become so if it
    /// acquired enough voting power.
    Inactive,
    /// The validator is an active part of the consensus set.
    Active,
    /// The validator has been removed from the consensus set, and all stake will finish unbonding
    /// at the epoch `unbonding_epoch`.
    Unbonding { unbonding_epoch: u64 },
    /// The validator has been slashed, and undelegations will occur immediately with no unbonding
    /// period.
    Slashed,
}

/// The name of a validator state, as a "C-style enum" without the extra information such as the
/// `unbonding_epoch`.
pub enum ValidatorStateName {
    /// The state name for [`ValidatorState::Inactive`].
    Inactive,
    /// The state name for [`ValidatorState::Active`].
    Active,
    /// The state name for [`ValidatorState::Unbonding`].
    Unbonding,
    /// The state name for [`ValidatorState::Slashed`].
    Slashed,
}

impl ValidatorState {
    /// Returns the name of the validator state.
    pub fn name(&self) -> ValidatorStateName {
        match self {
            ValidatorState::Inactive => ValidatorStateName::Inactive,
            ValidatorState::Active => ValidatorStateName::Active,
            ValidatorState::Unbonding { .. } => ValidatorStateName::Unbonding,
            ValidatorState::Slashed => ValidatorStateName::Slashed,
        }
    }
}

impl ValidatorStateName {
    /// Returns a static string representation of the validator state name.
    ///
    /// This is stable and should be used when serializing to strings (it is the inverse of [`FromStr::from_str`]).
    pub fn to_str(&self) -> &'static str {
        match self {
            ValidatorStateName::Inactive => "INACTIVE",
            ValidatorStateName::Active => "ACTIVE",
            ValidatorStateName::Unbonding => "UNBONDING",
            ValidatorStateName::Slashed => "SLASHED",
        }
    }
}

impl FromStr for ValidatorStateName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INACTIVE" => Ok(ValidatorStateName::Inactive),
            "ACTIVE" => Ok(ValidatorStateName::Active),
            "UNBONDING" => Ok(ValidatorStateName::Unbonding),
            "SLASHED" => Ok(ValidatorStateName::Slashed),
            _ => Err(anyhow::anyhow!("invalid validator state name: {}", s)),
        }
    }
}

impl From<ValidatorState> for (ValidatorStateName, Option<u64>) {
    fn from(state: ValidatorState) -> Self {
        match state {
            ValidatorState::Inactive => (ValidatorStateName::Inactive, None),
            ValidatorState::Active => (ValidatorStateName::Active, None),
            ValidatorState::Unbonding { unbonding_epoch } => {
                (ValidatorStateName::Unbonding, Some(unbonding_epoch))
            }
            ValidatorState::Slashed => (ValidatorStateName::Slashed, None),
        }
    }
}

impl TryFrom<(ValidatorStateName, Option<u64>)> for ValidatorState {
    type Error = anyhow::Error;

    fn try_from(state: (ValidatorStateName, Option<u64>)) -> Result<Self, Self::Error> {
        match state {
            (ValidatorStateName::Inactive, None) => Ok(ValidatorState::Inactive),
            (ValidatorStateName::Active, None) => Ok(ValidatorState::Active),
            (ValidatorStateName::Unbonding, Some(unbonding_epoch)) => {
                Ok(ValidatorState::Unbonding { unbonding_epoch })
            }
            (ValidatorStateName::Slashed, None) => Ok(ValidatorState::Slashed),
            (_, Some(_)) => Err(anyhow::anyhow!(
                "unbonding epoch not permitted with non-unbonding state"
            )),
            (ValidatorStateName::Unbonding, None) => Err(anyhow::anyhow!(
                "unbonding epoch not provided with unbonding state"
            )),
        }
    }
}
