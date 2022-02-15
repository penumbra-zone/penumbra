use std::collections::{BTreeMap, BTreeSet};

use anyhow::Error;
use penumbra_crypto::{note, Nullifier};
use penumbra_stake::ValidatorInfo;
use penumbra_transaction::{Action, Transaction};

use super::{NoteData, PendingTransaction, VerifiedTransaction};
use crate::state;

impl state::Reader {
    pub async fn verify_stateful(
        &self,
        transaction: PendingTransaction,
        // TODO: block_validators should just be a
        // Vec<Validator> or Vec<ValidatorDefinition> because we can't calculate
        // funding streams (necessary for ValidatorInfo)
        // until end_block
        block_validators: &[ValidatorInfo],
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

        // TODO: split into methods (after refactoring to have a single db query)

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
        // TODO: are any other checks necessary here?
        // TODO: what if multiple validators are added in the same block w/ same sequence numbers?
        // the winning definition will need to be selected deterministically. We could do this by sorting them
        // based on the signature bytes.
        for v in &transaction.validators {
            let existing_v: Vec<&ValidatorInfo> = block_validators
                .iter()
                .filter(|z| z.validator.identity_key == v.validator.identity_key)
                .collect();

            if existing_v.is_empty() {
                // This is a new validator definition.
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
        }

        Ok(VerifiedTransaction {
            id: transaction.id,
            new_notes: transaction.new_notes,
            spent_nullifiers: transaction.spent_nullifiers,
            delegation_changes,
            undelegation_validator: transaction.undelegation.map(|u| u.validator_identity),
            new_validators: transaction.validators,
        })
    }
}

// TODO: replace this with just inserting genesis notes directly

/// One-off function used to mark a genesis transaction as verified.
pub fn mark_genesis_as_verified(transaction: Transaction) -> VerifiedTransaction {
    let mut new_notes = BTreeMap::<note::Commitment, NoteData>::new();
    for action in transaction.transaction_body().actions {
        match action {
            Action::Output(inner) => {
                new_notes.insert(
                    inner.body.note_commitment,
                    NoteData {
                        ephemeral_key: inner.body.ephemeral_key,
                        encrypted_note: inner.body.encrypted_note,
                        transaction_id: transaction.id(),
                    },
                );
            }
            _ => {
                panic!("genesis transaction only has outputs")
            }
        }
    }

    VerifiedTransaction {
        id: transaction.id(),
        new_notes,
        spent_nullifiers: BTreeSet::<Nullifier>::new(),
        delegation_changes: BTreeMap::new(),
        undelegation_validator: None,
        new_validators: vec![],
    }
}
