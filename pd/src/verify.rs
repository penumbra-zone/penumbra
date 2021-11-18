use std::collections::BTreeSet;

use anyhow::Error;

use penumbra_crypto::{merkle, note, proofs, rdsa, value, Action, Nullifier, Transaction};

/// `PendingTransaction` holds data after stateless checks have been applied.
pub struct PendingTransaction {
    /// Root of the note commitment tree.
    pub root: merkle::Root,
    /// List of note commitments to add to the NCT from outputs in this transaction.
    pub new_notes: Vec<note::Commitment>,
    /// List of spent nullifiers from spends in this transaction.
    pub spent_nullifiers: BTreeSet<Nullifier>,
    /// List of spend proofs and their public inputs to verify in this transaction.
    pub spend_proof_details: Vec<SpendProofDetail>,
}

/// `VerifiedTransaction` represents a transaction after all checks have passed.
pub struct VerifiedTransaction {
    /// List of note commitments to add to the NCT from outputs in this transaction.
    pub new_notes: Vec<note::Commitment>,
    /// List of spent nullifiers from spends in this transaction.
    pub spent_nullifiers: BTreeSet<Nullifier>,
}

pub struct SpendProofDetail {
    pub proof: proofs::transparent::SpendProof,
    pub value_commitment: value::Commitment,
    pub nullifier: Nullifier,
    pub rk: rdsa::VerificationKey<rdsa::SpendAuth>,
}

pub trait StatelessTransactionExt {
    fn verify_stateless(&self) -> Result<PendingTransaction, Error>;
}

pub trait StatefulTransactionExt {
    fn verify_stateful(&self, nct_root: merkle::Root) -> Result<VerifiedTransaction, Error>;
}

impl StatelessTransactionExt for Transaction {
    fn verify_stateless(&self) -> Result<PendingTransaction, Error> {
        // 1. Check binding signature.
        if !self.verify_binding_sig() {
            return Err(anyhow::anyhow!("Binding signature did not verify"));
        }

        // 2. Check all spend auth signatures using provided spend auth keys
        // and check output proofs verify. If any action does not verify, the entire
        // transaction has failed.
        let mut spent_nullifiers = BTreeSet::<Nullifier>::new();
        // xx more stuff will go on here for Commit
        let mut new_notes = Vec::<note::Commitment>::new();
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

                    new_notes.push(inner.body.note_commitment);
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
            new_notes: self.new_notes.clone(),
            spent_nullifiers: self.spent_nullifiers.clone(),
        })
    }
}
