use penumbra_chain::CompactBlock;
use penumbra_crypto::Nullifier;
use penumbra_crypto::{
    merkle::{Frontier, Tree},
    FullViewingKey, Note, NotePayload,
};

use crate::NoteRecord;

/// Contains the results of scanning a single block.
pub struct ScanResult {
    // write as new rows
    pub new_notes: Vec<NoteRecord>,
    // use to update existing rows
    pub spent_nullifiers: Vec<Nullifier>,
    pub height: u64,
}

#[tracing::instrument(skip(fvk, note_commitment_tree, note_payloads, nullifiers))]
pub fn scan_block(
    fvk: &FullViewingKey,
    note_commitment_tree: &mut penumbra_crypto::merkle::NoteCommitmentTree,
    CompactBlock {
        height,
        note_payloads,
        nullifiers,
    }: CompactBlock,
) -> ScanResult {
    let mut new_notes: Vec<NoteRecord> = Vec::new();

    for NotePayload {
        note_commitment,
        ephemeral_key,
        encrypted_note,
    } in note_payloads
    {
        // Unconditionally insert the note commitment into the merkle tree
        tracing::debug!(?note_commitment, "appending to note commitment tree");
        note_commitment_tree.append(&note_commitment);

        // Try to decrypt the encrypted note using the ephemeral key and persistent incoming
        // viewing key -- if it doesn't decrypt, it wasn't meant for us.
        if let Ok(note) = Note::decrypt(encrypted_note.as_ref(), fvk.incoming(), &ephemeral_key) {
            tracing::debug!(?note_commitment, ?note, "found note while scanning");
            // Mark the most-recently-inserted note commitment (the one corresponding to this
            // note) as worth keeping track of, because it's ours
            note_commitment_tree.witness();

            // Insert the note associated with its computed nullifier into the nullifier map
            let (pos, _auth_path) = note_commitment_tree
                .authentication_path(&note_commitment)
                .expect("we just witnessed this commitment");

            let nullifier = fvk.derive_nullifier(pos, &note_commitment);

            let diversifier = &note.diversifier();

            let record = NoteRecord {
                note_commitment,
                height_spent: None,
                height_created: height,
                note,
                diversifier_index: fvk.incoming().index_for_diversifier(diversifier),
                nullifier,
            };

            new_notes.push(record);
        }
    }

    // TODO: write a query to mark all matching rows as spent

    ScanResult {
        new_notes,
        spent_nullifiers: nullifiers,
        height,
    }
}
