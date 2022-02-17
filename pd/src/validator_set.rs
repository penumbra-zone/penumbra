use std::{borrow::Borrow, collections::BTreeMap};

use anyhow::Result;

use tendermint::PublicKey;

use crate::state::Reader;
use penumbra_stake::{
    BaseRateData, Epoch, IdentityKey, RateData, Validator, ValidatorDefinition, ValidatorInfo,
    ValidatorState, ValidatorStatus,
};

#[derive(Debug, Clone)]
/// Records the complete state of all validators throughout a block,
/// and is responsible for producing the necessary database queries
/// for persistence.
///
/// After calling `BlockValidatorSet.commit`, the block is advanced.
/// Internal tracking of validator changes is reset for the new block.
pub struct BlockValidatorSet {
    /// Records complete validator states as they change during the course of the block.
    ///
    /// Updated as changes occur during the block, but will not be persisted
    /// to the database until the block is committed.
    validator_set: BTreeMap<IdentityKey, ValidatorInfo>,
    /// Database reader.
    reader: Reader,
    /// Validator definitions added during the block. Since multiple definitions could
    /// come in for the same validator during a block, we need to deterministically pick
    /// one definition to use.
    ///
    /// If the definition is for an existing validator, this will be pushed to `self.updated_validators`
    /// during end_block.
    ///
    /// Otherwise if the definition is for a new validator, this will be pushed to `self.new_validators`
    /// during end_block.
    pub validator_definitions: BTreeMap<IdentityKey, Vec<ValidatorDefinition>>,
    /// New validators added during the block. Saved and available for staking when the block is committed.
    pub new_validators: Vec<ValidatorInfo>,
    /// Existing validators updated during the block. Saved when the block is committed.
    pub updated_validators: Vec<ValidatorInfo>,
    /// Validators slashed during this block. Saved when the block is committed.
    ///
    /// The validator's rate will have a slashing penalty immediately applied during the current epoch.
    /// Their future rates will be held constant.
    pub slashed_validators: Vec<IdentityKey>,
    /// Indicates the epoch the block belongs to.
    pub epoch: Option<Epoch>,
    /// If this is the last block of an epoch, base rates for the next epoch go here.
    pub next_base_rate: Option<BaseRateData>,
    /// If this is the last block of an epoch, validator rates for the next epoch go here.
    pub next_rates: Option<Vec<RateData>>,
    /// If this is the last block of an epoch, validator statuses for the next epoch go here.
    pub next_validator_statuses: Option<Vec<ValidatorStatus>>,
}

impl BlockValidatorSet {
    pub async fn new(reader: Reader) -> Result<Self> {
        // Grab all validator info from the database. This will only happen when the
        // BlockValidatorSet is first instantiated.
        let block_validators = reader.validator_info(true).await?;

        // Initialize all state machine validator states to their current state from the block validators.
        let mut validator_set = BTreeMap::new();
        for validator in block_validators.iter() {
            validator_set.insert(validator.validator.identity_key.clone(), validator.clone());
        }

        Ok(BlockValidatorSet {
            validator_set,
            epoch: None,
            next_base_rate: None,
            next_rates: None,
            next_validator_statuses: None,
            validator_definitions: BTreeMap::new(),
            new_validators: Vec::new(),
            updated_validators: Vec::new(),
            slashed_validators: Vec::new(),
            reader,
        })
    }

    // Called during `end_block`. Responsible for resolving conflicting ValidatorDefinitions
    // that came in during the block and updating `validator_set` with new validator info.
    pub fn end_block(&mut self) {
        let make_validator = |v: ValidatorDefinition| -> ValidatorInfo {
            ValidatorInfo {
                validator: v.validator.clone(),
                // TODO: This is definitely wrong in the case of updated validator
                // definitions and would allow resetting state/rate data!
                //
                // These should be pulled from the existing validator if it exists!
                status: ValidatorStatus {
                    identity_key: v.validator.identity_key.clone(),
                    // Voting power for inactive validators is 0
                    voting_power: 0,
                    state: ValidatorState::Inactive,
                },
                rate_data: RateData {
                    identity_key: v.validator.identity_key,
                    epoch_index: self.epoch.as_ref().unwrap().index,
                    // Validator reward rate is held constant for inactive validators.
                    // Stake committed to inactive validators earns no rewards.
                    validator_reward_rate: 0,
                    // Exchange rate for inactive validators is held constant
                    // and starts at 1
                    validator_exchange_rate: 1,
                },
            }
        };

        // Any conflicts in validator definitions added to the pending block need to be resolved.
        for (ik, defs) in self.validator_definitions.iter_mut() {
            // Ensure the definitions are sorted by sequence number
            defs.sort_by(|a, b| {
                b.validator
                    .sequence_number
                    .cmp(&a.validator.sequence_number)
            });
            // TODO: Need to determine whether this is a new validator or an updated validator
            // and insert into the appropriate vec!
            if defs.len() == 1 {
                // If there was only one definition for an identity key, use it.
                self.new_validators.push(make_validator(defs[0].clone()));
                continue;
            }

            // Sort the validator definitions into buckets by their sequence number.
            let new_validator_definitions_by_seq =
                Vec::<(u32, Vec<ValidatorDefinition>)>::from_iter(
                    defs.iter()
                        .map(|def| (def.validator.sequence_number, vec![def.clone()])),
                );

            // The highest sequence number bucket wins.
            let highest_seq_bucket = &new_validator_definitions_by_seq[0];
        }
    }

    // Called during `end_epoch`. Will calculate validator changes that can only happen during epoch changes.
    pub fn end_epoch(&mut self) {}

    // TODO: this should *only* be called during `end_block`.
    pub fn add_validator(&mut self, validator: ValidatorInfo) {
        self.validator_set
            .insert(validator.validator.identity_key.clone(), validator);
    }

    pub fn add_validator_definition(&mut self, validator_definition: ValidatorDefinition) {
        let identity_key = validator_definition.validator.identity_key.clone();
        let validator_info = ValidatorInfo {
            validator: validator_definition.validator.clone(),
            // TODO: This is definitely wrong in the case of updated validator
            // definitions and would allow resetting state/rate data!
            //
            // These should be pulled from the existing validator if it exists!
            status: ValidatorStatus {
                identity_key: validator_definition.validator.identity_key.clone(),
                // Voting power for inactive validators is 0
                voting_power: 0,
                state: ValidatorState::Inactive,
            },
            rate_data: RateData {
                identity_key: validator_definition.validator.identity_key.clone(),
                epoch_index: self
                    .epoch
                    .as_ref()
                    .expect("expect epoch to be set when validator definitions are added")
                    .index,
                // Validator reward rate is held constant for inactive validators.
                // Stake committed to inactive validators earns no rewards.
                validator_reward_rate: 0,
                // Exchange rate for inactive validators is held constant
                // and starts at 1
                validator_exchange_rate: 1,
            },
        };
        // TODO: This might not be right, since the new validator definition
        // needs to be resolved during end_block. If multiple validators
        // are defined for the same sequence ID within a transaction, it
        // is possible that state could be overriden for a validator as a
        // result.
        //
        // For example:
        // Validator A is slashed during begin_block, but then an updated
        // ValidatorDefinition is submitted for Validator A in the same block.
        self.add_validator(validator_info.clone());
        self.validator_definitions
            .entry(identity_key)
            .or_insert_with(Vec::new)
            .push(validator_definition);
    }

    pub fn get_validator_info(&self, identity_key: &IdentityKey) -> Option<&ValidatorInfo> {
        self.validator_set.get(identity_key)
    }

    pub fn get_state(&self, identity_key: &IdentityKey) -> Option<&ValidatorState> {
        self.validator_set
            .get(identity_key)
            .map(|x| &x.status.state)
    }

    pub fn validators(&self) -> impl Iterator<Item = impl Borrow<&'_ Validator>> {
        self.validator_set.iter().map(|v| &v.1.validator)
    }

    pub fn validators_info(&self) -> impl Iterator<Item = impl Borrow<&'_ ValidatorInfo>> {
        self.validator_set.iter().map(|v| v.1)
    }

    /// Returns all validators that are currently in the `Slashed` state.
    pub fn slashed_validators(&self) -> impl Iterator<Item = impl Borrow<&'_ Validator>> {
        self.validator_set
            .iter()
            .filter(|v| v.1.status.state == ValidatorState::Slashed)
            .map(|v| &v.1.validator)
    }

    pub fn unslashed_validators(&self) -> impl Iterator<Item = impl Borrow<&'_ Validator>> {
        // validators: Option<impl IntoIterator<Item = impl Borrow<&'a IdentityKey>>>,
        self.validator_set
            .iter()
            // Return all validators that are *not slashed*
            .filter(|v| v.1.status.state != ValidatorState::Slashed)
            .map(|v| &v.1.validator)
    }

    /// Mark a validator as deactivated. Only validators in the unbonding state
    /// can be deactivated.
    pub fn deactivate_validator(&mut self, ck: PublicKey) -> Result<()> {
        // Don't love this clone.
        let validator = self.get_validator_by_consensus_key(&ck)?.clone();

        let current_info = self
            .get_validator_info(&validator.identity_key)
            .ok_or(anyhow::anyhow!("Validator not found in state machine"))?;
        let current_state = current_info.status.state;

        match current_state {
            ValidatorState::Unbonding { unbonding_epoch: _ } => {
                self.validator_set
                    .get_mut(&validator.identity_key)
                    .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                    .status
                    .state = ValidatorState::Inactive;
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Validator {} is not in unbonding state",
                validator.identity_key
            )),
        }
    }

    // Activate a validator. Only validators in the inactive or unbonding state
    // may be activated.
    pub fn activate_validator(&mut self, ck: PublicKey) -> Result<()> {
        // Don't love this clone.
        let validator = self.get_validator_by_consensus_key(&ck)?.clone();

        let current_info = self
            .get_validator_info(&validator.identity_key)
            .ok_or(anyhow::anyhow!("Validator not found in state machine"))?;
        let current_state = current_info.status.state;

        let mut mark_active = |validator: &Validator| -> Result<()> {
            self.validator_set
                .get_mut(&validator.identity_key)
                .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                .status
                .state = ValidatorState::Active;
            Ok(())
        };

        match current_state {
            ValidatorState::Inactive => mark_active(&validator),
            // The unbonding epoch is not checked here. It is checked in the
            // consensus worker.
            ValidatorState::Unbonding { unbonding_epoch: _ } => mark_active(&validator),
            _ => Err(anyhow::anyhow!(
                "only validators in the inactive or unbonding state may be activated"
            )),
        }
    }

    // Marks a validator as slashed. Only validators in the active or unbonding
    // state may be slashed.
    pub fn slash_validator(&mut self, ck: &PublicKey, slashing_penalty: u64) -> Result<()> {
        // Don't love this clone.
        let validator = self.get_validator_by_consensus_key(ck)?.clone();

        self.slashed_validators.push(validator.identity_key.clone());

        let current_info = self
            .get_validator_info(&validator.identity_key)
            .ok_or(anyhow::anyhow!("Validator not found in state machine"))?;
        let current_state = current_info.status.state;

        let mut mark_slashed = |validator: &Validator| -> Result<()> {
            self.validator_set
                .get_mut(&validator.identity_key)
                .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                .status
                .state = ValidatorState::Slashed;
            self.validator_set
                .get_mut(&validator.identity_key)
                .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                .rate_data
                // TODO: pretty sure this calculation of slashed rate is wrong
                .validator_reward_rate -= slashing_penalty;
            Ok(())
        };

        match current_state {
            ValidatorState::Active => mark_slashed(&validator),
            ValidatorState::Unbonding { unbonding_epoch: _ } => mark_slashed(&validator),
            _ => Err(anyhow::anyhow!(
                "only validators in the active or unbonding state may be slashed"
            )),
        }
    }

    // Marks a validator as unbonding. Only validators in the active state
    // may begin unbonding.
    pub fn unbond_validator(&mut self, ck: PublicKey, unbonding_epoch: u64) -> Result<()> {
        // Don't love this clone.
        let validator = self.get_validator_by_consensus_key(&ck)?.clone();
        let current_info = self
            .get_validator_info(&validator.identity_key)
            .ok_or(anyhow::anyhow!("Validator not found in state machine"))?;
        let current_state = current_info.status.state;

        match current_state {
            ValidatorState::Active => {
                // Unbonding the validator means that it can no longer participate
                // in consensus, so its voting power is set to 0.
                self.validator_set
                    .get_mut(&validator.identity_key)
                    .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                    .status
                    .voting_power = 0;
                self.validator_set
                    .get_mut(&validator.identity_key)
                    .ok_or_else(|| anyhow::anyhow!("Validator not found"))?
                    .status
                    .state = ValidatorState::Unbonding { unbonding_epoch };
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "only validators in the active state may begin unbonding"
            )),
        }
    }

    // Tendermint validators are referenced to us by their Tendermint consensus key,
    // but we reference them by their Penumbra identity key.
    pub fn get_validator_by_consensus_key(&self, ck: &PublicKey) -> Result<&Validator> {
        let validator = self
            .validator_set
            .iter()
            .find(|v| v.1.validator.consensus_key == *ck)
            .ok_or(anyhow::anyhow!("No validator found"))?;
        Ok(&validator.1.validator)
    }
}
