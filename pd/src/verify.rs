use std::collections::{BTreeMap, BTreeSet};

use anyhow::Error;

use penumbra_crypto::{ka, merkle, note, proofs, rdsa, value, Action, Nullifier, Transaction};

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
    /// List of spend proofs and their public inputs to verify in this transaction.
    pub spend_proof_details: Vec<SpendProofDetail>,
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

pub struct SpendProofDetail {
    pub proof: proofs::transparent::SpendProof,
    pub value_commitment: value::Commitment,
    pub nullifier: Nullifier,
    pub rk: rdsa::VerificationKey<rdsa::SpendAuth>,
}

pub trait StatelessTransactionExt {
    fn verify_stateless(
        &self,
        value_balance: Option<decaf377::Element>,
    ) -> Result<PendingTransaction, Error>;
}

pub trait StatefulTransactionExt {
    fn verify_stateful(&self, nct_root: merkle::Root) -> Result<VerifiedTransaction, Error>;
}

impl StatelessTransactionExt for Transaction {
    fn verify_stateless(
        &self,
        value_balance: Option<decaf377::Element>,
    ) -> Result<PendingTransaction, Error> {
        let id = self.id();

        // 1. Check binding signature, using the value_balance if provided.
        match value_balance {
            Some(value_balance) => {
                if !self.verify_binding_sig_with_value_balance(value_balance) {
                    return Err(anyhow::anyhow!("Binding signature did not verify"));
                }
            }
            None => {
                if !self.verify_binding_sig() {
                    return Err(anyhow::anyhow!("Binding signature did not verify"));
                }
            }
        }

        // 2. Check all spend auth signatures using provided spend auth keys
        // and check output proofs verify. If any action does not verify, the entire
        // transaction has failed.
        let mut spent_nullifiers = BTreeSet::<Nullifier>::new();
        let mut new_notes = BTreeMap::<note::Commitment, NoteData>::new();
        let mut spend_proof_details = Vec::<SpendProofDetail>::new();

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

                    // We save this data and verify later as it requires the `App` NCT state.
                    spend_proof_details.push(SpendProofDetail {
                        proof: inner.body.proof,
                        value_commitment: inner.body.value_commitment,
                        nullifier: inner.body.nullifier.clone(),
                        rk: inner.body.rk,
                    });

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
            spend_proof_details,
        })
    }
}

impl StatefulTransactionExt for PendingTransaction {
    fn verify_stateful(&self, nct_root: merkle::Root) -> Result<VerifiedTransaction, Error> {
        for spend in &self.spend_proof_details {
            if !spend.proof.verify(
                nct_root.clone(),
                spend.value_commitment,
                spend.nullifier.clone(),
                spend.rk,
            ) {
                return Err(anyhow::anyhow!("Spend proof does not verify"));
            }
        }

        Ok(VerifiedTransaction {
            id: self.id,
            new_notes: self.new_notes.clone(),
            spent_nullifiers: self.spent_nullifiers.clone(),
        })
    }
}
