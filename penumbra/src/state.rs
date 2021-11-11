use std::collections::HashMap;
use std::convert::TryInto;

use penumbra_crypto::{ka, note, Action, Transaction};

/// Stores pending state changes from transactions.
#[derive(Debug)]
pub struct PendingBlock {
    // Stores note commitments for convienience when updating the NCT.
    pub note_commitments: Vec<note::Commitment>,
    // A map of serialized transactions (what we store) to its notes.
    pub transactions: HashMap<Vec<u8>, Vec<NoteFragment>>,
}

impl Default for PendingBlock {
    fn default() -> Self {
        PendingBlock {
            note_commitments: Vec::new(),
            transactions: HashMap::new(),
        }
    }
}

impl PendingBlock {
    /// Adds the changes from a transaction.
    pub fn add_transaction(&mut self, transaction: Transaction) {
        let mut note_fragments = Vec::<NoteFragment>::new();
        for action in transaction.transaction_body().actions {
            match action {
                Action::Output(output) => {
                    // Unpack new notes from outputs, save here.
                    note_fragments.push(NoteFragment {
                        note_commitment: output.body.note_commitment,
                        ephemeral_key: output.body.ephemeral_key,
                        note_ciphertext: output.body.encrypted_note,
                    });
                    self.note_commitments.push(output.body.note_commitment);
                }
                Action::Spend(_spend) => {
                    // This should be done when implementing `DeliverTx`
                    todo!("add nullifiers from spends to the database!")
                }
            }
        }

        let serialized_transaction: Vec<u8> = transaction
            .try_into()
            .expect("can serialize genesis transaction");

        self.transactions
            .insert(serialized_transaction, note_fragments);
    }
}

#[derive(Debug)]
pub struct NoteFragment {
    pub note_commitment: note::Commitment,
    pub ephemeral_key: ka::Public,
    pub note_ciphertext: [u8; note::NOTE_CIPHERTEXT_BYTES],
}
