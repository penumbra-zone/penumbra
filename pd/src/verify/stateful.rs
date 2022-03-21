use std::{borrow::Borrow, collections::BTreeMap};

use anyhow::Error;
use penumbra_stake::{ValidatorInfo, ValidatorState};

use super::{PendingTransaction, VerifiedTransaction};
use crate::state;

impl state::Reader {
    pub async fn verify_stateful<'a, T: Clone + Iterator<Item = impl Borrow<&'a ValidatorInfo>>>(
        &self,
        transaction: PendingTransaction,
        // TODO: taking a BTreeMap here would let us avoid linear search times later on
        // We can't take a `ValidatorSet` because it's also called during `check_tx` and
        // the mempool worker doesn't have access to the consensus worker's validator set.
        block_validators: T,
    ) -> Result<VerifiedTransaction, Error> {
        let anchor_is_valid = self.valid_anchors_rx().borrow().contains(&transaction.root);
        if !anchor_is_valid {
            return Err(anyhow::anyhow!("invalid note commitment tree root"));
        }

        let existing_nullifiers = self.check_nullifiers(&transaction.spent_nullifiers).await?;
        if !existing_nullifiers.is_empty() {
            return Err(anyhow::anyhow!(
                "nullifiers already spent in state: {:?}",
                existing_nullifiers
            ));
        }

        // TODO: split into methods

        // Tally the delegations and undelegations
        let mut delegation_changes = BTreeMap::new();
        for d in &transaction.delegations {
            let rate_data = self
                .next_rate_data_rx()
                .borrow()
                .get(&d.validator_identity)
                .ok_or_else(|| {
                    anyhow::anyhow!("Unknown validator identity {}", d.validator_identity)
                })?
                .clone();

            // Check whether the epoch is correct first, to give a more helpful
            // error message if it's wrong.
            if d.epoch_index != rate_data.epoch_index {
                return Err(anyhow::anyhow!(
                    "Delegation was prepared for next epoch {} but the next epoch is {}",
                    d.epoch_index,
                    rate_data.epoch_index
                ));
            }

            // Check whether the delegation is for a slashed validator
            if let Some(v) = block_validators
                .clone()
                .find(|v| v.borrow().validator.identity_key == d.validator_identity)
            {
                if v.borrow().status.state == ValidatorState::Slashed {
                    return Err(anyhow::anyhow!(
                        "Delegation to slashed validator {}",
                        d.validator_identity
                    ));
                }
            };

            // For delegations, we enforce correct computation (with rounding)
            // of the *delegation amount based on the unbonded amount*, because
            // users (should be) starting with the amount of unbonded stake they
            // wish to delegate, and computing the amount of delegation tokens
            // they receive.
            //
            // The direction of the computation matters because the computation
            // involves rounding, so while both
            //
            // (unbonded amount, rates) -> delegation amount
            // (delegation amount, rates) -> unbonded amount
            //
            // should give approximately the same results, they may not give
            // exactly the same results.
            let expected_delegation_amount = rate_data.delegation_amount(d.unbonded_amount);

            if expected_delegation_amount == d.delegation_amount {
                // The delegation amount is added to the delegation token supply.
                *delegation_changes
                    .entry(d.validator_identity.clone())
                    .or_insert(0) += i64::try_from(d.delegation_amount).unwrap();
            } else {
                return Err(anyhow::anyhow!(
                    "Given {} unbonded stake, expected {} delegation tokens but description produces {}",
                    d.unbonded_amount,
                    expected_delegation_amount,
                    d.delegation_amount
                ));
            }
        }
        if let Some(ref u) = transaction.undelegation {
            let rate_data = self
                .next_rate_data_rx()
                .borrow()
                .get(&u.validator_identity)
                .ok_or_else(|| {
                    anyhow::anyhow!("Unknown validator identity {}", u.validator_identity)
                })?
                .clone();

            // Check whether the epoch is correct first, to give a more helpful
            // error message if it's wrong.
            if u.epoch_index != rate_data.epoch_index {
                return Err(anyhow::anyhow!(
                    "Undelegation was prepared for next epoch {} but the next epoch is {}",
                    u.epoch_index,
                    rate_data.epoch_index
                ));
            }

            // For undelegations, we enforce correct computation (with rounding)
            // of the *unbonded amount based on the delegation amount*, because
            // users (should be) starting with the amount of delegation tokens they
            // wish to undelegate, and computing the amount of unbonded stake
            // they receive.
            //
            // The direction of the computation matters because the computation
            // involves rounding, so while both
            //
            // (unbonded amount, rates) -> delegation amount
            // (delegation amount, rates) -> unbonded amount
            //
            // should give approximately the same results, they may not give
            // exactly the same results.
            let expected_unbonded_amount = rate_data.unbonded_amount(u.delegation_amount);

            if expected_unbonded_amount == u.unbonded_amount {
                // TODO: in order to have exact tracking of the token supply, we probably
                // need to change this to record the changes to the unbonded stake and
                // the delegation token separately

                // The undelegation amount is subtracted from the delegation token supply.
                *delegation_changes
                    .entry(u.validator_identity.clone())
                    .or_insert(0) -= i64::try_from(u.delegation_amount).unwrap();
            } else {
                return Err(anyhow::anyhow!(
                    "Given {} delegation tokens, expected {} unbonded stake but description produces {}",
                    u.delegation_amount,
                    expected_unbonded_amount,
                    u.unbonded_amount,
                ));
            }
        }

        // Check that the sequence numbers of newly added validators are correct.
        //
        // Resolution of conflicting validator definitions is performed later in `end_block` after
        // they've all been received.
        let mut validator_definitions = Vec::new();
        for v in &transaction.validator_definitions {
            let existing_v: Vec<&ValidatorInfo> = block_validators
                .clone()
                .filter(|z| z.borrow().validator.identity_key == v.validator.identity_key)
                // vvv that looks weird to me and seems like a potential anti-pattern
                .map(|z| *z.borrow())
                .collect();

            if existing_v.is_empty() {
                // This is a new validator definition.
                validator_definitions.push(v.clone().into());
                continue;
            } else {
                // This is an existing validator definition. Ensure that the highest
                // existing sequence number is less than the new sequence number.
                let current_seq = existing_v.iter().map(|z| z.validator.sequence_number).max().ok_or_else(|| {anyhow::anyhow!("Validator with this ID key existed but had no existing sequence numbers")})?;
                if v.validator.sequence_number <= current_seq {
                    return Err(anyhow::anyhow!(
                        "Expected sequence numbers to be increasing. Current sequence number is {}",
                        current_seq
                    ));
                }
            }

            // the validator definition has now passed all verification checks, so add it to the list
            validator_definitions.push(v.clone().into());
        }

        Ok(VerifiedTransaction {
            id: transaction.id,
            new_notes: transaction.new_notes,
            spent_nullifiers: transaction.spent_nullifiers,
            delegation_changes,
            undelegation_validator: transaction.undelegation.map(|u| u.validator_identity),
            validator_definitions,
        })
    }
}
