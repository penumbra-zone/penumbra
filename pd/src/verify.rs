use std::collections::BTreeSet;

use anyhow::Error;

use penumbra_crypto::{merkle, note, Action, Nullifier, Transaction};

/// `PendingTransaction` holds data after stateless checks have been applied.
pub struct PendingTransaction {
    /// Root of the note commitment tree.
    pub root: merkle::Root,
    /// List of note commitments to add to the NCT from outputs in this transaction.
    pub new_notes: Vec<note::Commitment>,
    /// List of spent nullifiers from spends in this transaction.
    pub spent_nullfiers: BTreeSet<Nullifier>,
}

trait TransactionExt {
    fn verify_stateless(&self) -> Result<PendingTransaction, Error>;
}

impl TransactionExt for Transaction {
    fn verify_stateless(&self) -> Result<PendingTransaction, Error> {
        // 1. Check binding signature.
        if !self.verify_binding_sig() {
            return Err(anyhow::anyhow!("Binding signature did not verify"));
        }

        // 2. Check all spend auth signatures using provided spend auth keys
        // and check all proofs verify. If any action does not verify, the entire
        // transaction has failed.
        let mut nullifiers_to_add = BTreeSet::<Nullifier>::new();
        let mut note_commitments_to_add = Vec::<note::Commitment>::new();

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

                    note_commitments_to_add.push(inner.body.note_commitment);
                }
                Action::Spend(inner) => {
                    if !inner.verify_auth_sig() {
                        return Err(anyhow::anyhow!("A spend auth sig did not verify"));
                    }

                    // TODO: The below check is stateful and should go elsewhere
                    // if !inner.body.proof.verify(
                    //     self.note_commitment_tree.root2(),
                    //     inner.body.value_commitment,
                    //     inner.body.nullifier.clone(),
                    //     inner.body.rk,
                    // ) {
                    //     return false;
                    // }

                    // Check nullifier is not already in the nullifier set OR
                    // has been revealed already in this transaction.
                    /*
                    if self.nullifier_set.contains(&inner.body.nullifier.clone())
                        || nullifiers_to_add.contains(&inner.body.nullifier.clone())
                    {
                        return false;
                    }
                    */

                    nullifiers_to_add.insert(inner.body.nullifier.clone());
                }
            }
        }

        Ok(PendingTransaction {
            root: self.transaction_body().merkle_root,
            new_notes: note_commitments_to_add,
            spent_nullfiers: nullifiers_to_add,
        })
    }
}
