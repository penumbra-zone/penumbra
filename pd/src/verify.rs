use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::Error;

use penumbra_crypto::{ka, merkle, note, Action, Nullifier, Transaction};

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
}

/// `VerifiedTransaction` represents a transaction after all checks have passed.
pub struct VerifiedTransaction {
    /// Transaction ID.
    pub id: [u8; 32],
    /// Note data to add from outputs in this transaction.
    pub new_notes: BTreeMap<note::Commitment, NoteData>,
    /// List of spent nullifiers from spends in this transaction.
    pub spent_nullifiers: BTreeSet<Nullifier>,
}

#[derive(Debug, Clone)]
pub struct NoteData {
    pub ephemeral_key: ka::Public,
    pub encrypted_note: [u8; note::NOTE_CIPHERTEXT_BYTES],
    pub transaction_id: [u8; 32],
}

pub trait StatelessTransactionExt {
    fn verify_stateless(&self) -> Result<PendingTransaction, Error>;
}

pub trait StatefulTransactionExt {
    fn verify_stateful(
        &self,
        valid_anchors: &VecDeque<merkle::Root>,
    ) -> Result<VerifiedTransaction, Error>;
}

impl StatelessTransactionExt for Transaction {
    fn verify_stateless(&self) -> Result<PendingTransaction, Error> {
        let id = self.id();

        // 1. Check binding signature.
        if !self.verify_binding_sig() {
            return Err(anyhow::anyhow!("Binding signature did not verify"));
        }

        // 2. Check all spend auth signatures using provided spend auth keys
        // and check all proofs verify. If any action does not verify, the entire
        // transaction has failed.
        let mut spent_nullifiers = BTreeSet::<Nullifier>::new();
        let mut new_notes = BTreeMap::<note::Commitment, NoteData>::new();

        for action in self.transaction_body().actions {
            match action {
                Action::Output(inner) => {
                    if !inner.body.proof.verify(
                        inner.body.value_commitment,
                        inner.body.note_commitment,
                        inner.body.ephemeral_key,
                    ) {
                        return Err(anyhow::anyhow!("An output proof did not verify"));
                    }

                    new_notes.insert(
                        inner.body.note_commitment,
                        NoteData {
                            ephemeral_key: inner.body.ephemeral_key,
                            encrypted_note: inner.body.encrypted_note,
                            transaction_id: id,
                        },
                    );
                }
                Action::Spend(inner) => {
                    if !inner.verify_auth_sig() {
                        return Err(anyhow::anyhow!("A spend auth sig did not verify"));
                    }

                    if !inner.body.proof.verify(
                        self.transaction_body().merkle_root,
                        inner.body.value_commitment,
                        inner.body.nullifier.clone(),
                        inner.body.rk,
                    ) {
                        return Err(anyhow::anyhow!("A spend proof did not verify"));
                    }

                    // Check nullifier has not been revealed already in this transaction.
                    if spent_nullifiers.contains(&inner.body.nullifier.clone()) {
                        return Err(anyhow::anyhow!("Double spend"));
                    }

                    spent_nullifiers.insert(inner.body.nullifier.clone());
                }
            }
        }

        Ok(PendingTransaction {
            id,
            root: self.transaction_body().merkle_root,
            new_notes,
            spent_nullifiers,
        })
    }
}

impl StatefulTransactionExt for PendingTransaction {
    fn verify_stateful(
        &self,
        valid_anchors: &VecDeque<merkle::Root>,
    ) -> Result<VerifiedTransaction, Error> {
        if !valid_anchors.contains(&self.root) {
            return Err(anyhow::anyhow!("invalid note commitment tree root"));
        }

        Ok(VerifiedTransaction {
            id: self.id,
            new_notes: self.new_notes.clone(),
            spent_nullifiers: self.spent_nullifiers.clone(),
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
            Action::Spend(_) => {
                panic!("genesis transaction has no spends")
            }
        }
    }

    VerifiedTransaction {
        id: transaction.id(),
        new_notes,
        spent_nullifiers: BTreeSet::<Nullifier>::new(),
    }
}
