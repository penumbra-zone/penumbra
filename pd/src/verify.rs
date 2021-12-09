use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Error};

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

#[derive(Debug, Clone)]
pub struct PositionedNoteData {
    pub position: u64,
    pub data: NoteData,
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

#[cfg(test)]
mod tests {
    use super::*;

    use ark_ff::Zero;
    use penumbra_crypto::{
        asset,
        keys::SpendKey,
        memo::MemoPlaintext,
        merkle::{Frontier, Tree, TreeExt},
        Fq, Note, Value,
    };
    use rand_core::OsRng;

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
            asset_id: asset::Denom::from("penumbra").into(),
        };
        let spend_value = Value {
            amount: 20,
            asset_id: asset::Denom::from("penumbra").into(),
        };
        // The note was previously sent to the sender.
        let note = Note::new(
            *send_addr.diversifier(),
            *send_addr.transmission_key(),
            spend_value,
            Fq::zero(),
        )
        .expect("transmission key is valid");
        let note_commitment = note.commit();

        let mut nct = merkle::BridgeTree::<note::Commitment, 32>::new(1);
        nct.append(&note_commitment);
        let anchor = nct.root2();
        nct.witness();
        let auth_path = nct.authentication_path(&note_commitment).unwrap();
        let merkle_path = (u64::from(auth_path.0) as usize, auth_path.1);

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
            .add_spend(&mut rng, sk_sender, merkle_path, note, auth_path.0)
            .finalize(&mut rng)
            .expect("transaction created ok");

        let pending_tx = transaction
            .verify_stateless()
            .expect("stateless verification should pass");

        let mut valid_anchors: VecDeque<merkle::Root> = VecDeque::new();
        valid_anchors.push_back(anchor);

        let _verified_tx = pending_tx
            .verify_stateful(&valid_anchors)
            .expect("stateful verification should pass");
    }
}
