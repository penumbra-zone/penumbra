use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Error};
use penumbra_crypto::{ka, merkle, note, Nullifier};
use penumbra_stake::{Delegate, IdentityKey, RateData, Undelegate};
use penumbra_transaction::{Action, Transaction};

/// `PendingTransaction` holds data after stateless checks have been applied.
pub struct PendingTransaction {
    /// Transaction ID.
    pub id: [u8; 32],
    /// Root of the note commitment tree.
    pub root: merkle::Root,
    /// Note data to add from outputs in this transaction.
    pub new_notes: BTreeMap<note::Commitment, NoteData>,
    /// List of spent nullifiers from spends in this transaction.
    pub spent_nullifiers: BTreeSet<Nullifier>,
    /// Delegations performed in this transaction.
    pub delegations: Vec<Delegate>,
    /// Undelegations performed in this transaction.
    pub undelegations: Vec<Undelegate>,
}

/// `VerifiedTransaction` represents a transaction after all checks have passed.
pub struct VerifiedTransaction {
    /// Transaction ID.
    pub id: [u8; 32],
    /// Note data to add from outputs in this transaction.
    pub new_notes: BTreeMap<note::Commitment, NoteData>,
    /// List of spent nullifiers from spends in this transaction.
    pub spent_nullifiers: BTreeSet<Nullifier>,
    /// Net delegations performed in this transaction.
    ///
    /// An identity key mapped to zero is different from an identity key that is absent; the former
    /// indicates that a validator's net change in delegation in this transaction was zero *but it
    /// experienced some (un)delegations*.
    pub delegation_changes: BTreeMap<IdentityKey, i64>,
    /// The validators from whom an undelegation was performed in this transaction.
    pub undelegation_validators: BTreeSet<IdentityKey>,
}

#[derive(Debug, Clone)]
pub struct NoteData {
    pub ephemeral_key: ka::Public,
    pub encrypted_note: [u8; note::NOTE_CIPHERTEXT_BYTES],
    pub transaction_id: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct PositionedNoteData {
    pub position: u64,
    pub data: NoteData,
}

#[derive(Debug, Clone)]
pub struct QuarantinedNoteData {
    pub validator_identity_keys: BTreeSet<IdentityKey>,
    pub data: NoteData,
}

pub trait StatelessTransactionExt {
    fn verify_stateless(&self) -> Result<PendingTransaction, Error>;
}

pub trait StatefulTransactionExt {
    fn verify_stateful(
        &self,
        valid_anchors: &VecDeque<merkle::Root>,
        next_rate_data: &BTreeMap<IdentityKey, RateData>,
    ) -> Result<VerifiedTransaction, Error>;
}

impl StatelessTransactionExt for Transaction {
    fn verify_stateless(&self) -> Result<PendingTransaction, Error> {
        let id = self.id();

        let sighash = self.transaction_body().sighash();

        // 1. Check binding signature.
        self.binding_verification_key()
            .verify(&sighash, self.binding_sig())
            .context("binding signature failed to verify")?;

        // 2. Check all spend auth signatures using provided spend auth keys
        // and check all proofs verify. If any action does not verify, the entire
        // transaction has failed.
        let mut spent_nullifiers = BTreeSet::<Nullifier>::new();
        let mut new_notes = BTreeMap::<note::Commitment, NoteData>::new();
        let mut delegations = Vec::<Delegate>::new();
        let mut undelegations = Vec::<Undelegate>::new();

        for action in self.transaction_body().actions {
            match action {
                Action::Output(output) => {
                    if output
                        .body
                        .proof
                        .verify(
                            output.body.value_commitment,
                            output.body.note_commitment,
                            output.body.ephemeral_key,
                        )
                        .is_err()
                    {
                        // TODO should the verification error be bubbled up here?
                        return Err(anyhow::anyhow!("An output proof did not verify"));
                    }

                    new_notes.insert(
                        output.body.note_commitment,
                        NoteData {
                            ephemeral_key: output.body.ephemeral_key,
                            encrypted_note: output.body.encrypted_note,
                            transaction_id: id,
                        },
                    );
                }
                Action::Spend(spend) => {
                    spend
                        .body
                        .rk
                        .verify(&sighash, &spend.auth_sig)
                        .context("spend auth signature failed to verify")?;

                    if spend
                        .body
                        .proof
                        .verify(
                            self.transaction_body().merkle_root,
                            spend.body.value_commitment,
                            spend.body.nullifier.clone(),
                            spend.body.rk,
                        )
                        .is_err()
                    {
                        // TODO should the verification error be bubbled up here?
                        return Err(anyhow::anyhow!("A spend proof did not verify"));
                    }

                    // Check nullifier has not been revealed already in this transaction.
                    if spent_nullifiers.contains(&spend.body.nullifier.clone()) {
                        return Err(anyhow::anyhow!("Double spend"));
                    }

                    spent_nullifiers.insert(spend.body.nullifier.clone());
                }
                Action::Delegate(delegate) => {
                    // There are currently no stateless verification checks than the ones implied by
                    // the binding signature.
                    delegations.push(delegate);
                }
                Action::Undelegate(undelegate) => {
                    // There are currently no stateless verification checks than the ones implied by
                    // the binding signature.
                    undelegations.push(undelegate);
                }
                _ => todo!("unsupported action"),
            }
        }

        Ok(PendingTransaction {
            id,
            root: self.transaction_body().merkle_root,
            new_notes,
            spent_nullifiers,
            delegations,
            undelegations,
        })
    }
}

impl StatefulTransactionExt for PendingTransaction {
    fn verify_stateful(
        &self,
        valid_anchors: &VecDeque<merkle::Root>,
        next_rate_data: &BTreeMap<IdentityKey, RateData>,
    ) -> Result<VerifiedTransaction, Error> {
        if !valid_anchors.contains(&self.root) {
            return Err(anyhow::anyhow!("invalid note commitment tree root"));
        }

        // Tally the delegations and undelegations
        let mut delegation_changes = BTreeMap::new();
        for d in &self.delegations {
            let rate_data = next_rate_data.get(&d.validator_identity).ok_or_else(|| {
                anyhow::anyhow!("Unknown validator identity {}", d.validator_identity)
            })?;

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
        for u in &self.undelegations {
            let rate_data = next_rate_data.get(&u.validator_identity).ok_or_else(|| {
                anyhow::anyhow!("Unknown validator identity {}", u.validator_identity)
            })?;

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

        Ok(VerifiedTransaction {
            id: self.id,
            new_notes: self.new_notes.clone(),
            spent_nullifiers: self.spent_nullifiers.clone(),
            delegation_changes,
            undelegation_validators: self
                .undelegations
                .iter()
                .map(|u| u.validator_identity.clone())
                .collect(),
        })
    }
}

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
        undelegation_validators: BTreeSet::new(),
    }
}

#[cfg(test)]
mod tests {
    use ark_ff::Zero;
    use penumbra_crypto::{
        asset,
        keys::SpendKey,
        memo::MemoPlaintext,
        merkle::{Frontier, NoteCommitmentTree, Tree, TreeExt},
        Fq, Note, Value,
    };
    use rand_core::OsRng;

    use super::*;

    #[test]
    fn test_transaction_succeeds_if_values_balance() {
        let mut rng = OsRng;
        let sk_sender = SpendKey::generate(&mut rng);
        let fvk_sender = sk_sender.full_viewing_key();
        let ovk_sender = fvk_sender.outgoing();
        let (send_addr, _) = fvk_sender.incoming().payment_address(0u64.into());

        let sk_recipient = SpendKey::generate(&mut rng);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let output_value = Value {
            amount: 10,
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let spend_value = Value {
            amount: 20,
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        // The note was previously sent to the sender.
        let note = Note::from_parts(
            *send_addr.diversifier(),
            *send_addr.transmission_key(),
            spend_value,
            Fq::zero(),
        )
        .expect("transmission key is valid");
        let note_commitment = note.commit();

        let mut nct = NoteCommitmentTree::new(1);
        nct.append(&note_commitment);
        nct.witness();
        let anchor = nct.root2();

        let transaction = Transaction::build_with_root(anchor.clone())
            .set_fee(10)
            .set_chain_id("penumbra".to_string())
            .add_output(
                &mut rng,
                &dest,
                output_value,
                MemoPlaintext::default(),
                ovk_sender,
            )
            .add_spend(&mut rng, &nct, &sk_sender, note)
            .expect("note is in nct")
            .finalize(&mut rng)
            .expect("transaction created ok");

        let pending_tx = transaction
            .verify_stateless()
            .expect("stateless verification should pass");

        let mut valid_anchors: VecDeque<merkle::Root> = VecDeque::new();
        valid_anchors.push_back(anchor);

        let _verified_tx = pending_tx
            .verify_stateful(&valid_anchors, &BTreeMap::default())
            .expect("stateful verification should pass");
    }
}
