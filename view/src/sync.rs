use std::collections::BTreeMap;

use penumbra_chain::{CompactBlock, Epoch};
use penumbra_crypto::{note, Nullifier};
use penumbra_crypto::{FullViewingKey, Note, NotePayload};
use penumbra_tct as tct;

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
    note_commitment_tree: &mut tct::Tree,
    CompactBlock {
        height,
        note_payloads,
        nullifiers,
        block_root,
        epoch_root,
        quarantined_note_payloads,
        quarantined_nullifiers,
        rolled_back_note_commitments,
        rolled_back_nullifiers,
    }: CompactBlock,
    epoch_duration: u64,
) -> ScanResult {
    // Trial-decrypt the notes in this block, keeping track of the ones that were meant for us
    let mut decrypted_notes: BTreeMap<note::Commitment, Note> = note_payloads
        .iter()
        .filter_map(
            |NotePayload {
                 note_commitment,
                 ephemeral_key,
                 encrypted_note,
             }| {
                // Try to decrypt the encrypted note using the ephemeral key and persistent incoming
                // viewing key -- if it doesn't decrypt, it wasn't meant for us.
                if let Ok(note) =
                    Note::decrypt(encrypted_note.as_ref(), fvk.incoming(), ephemeral_key)
                {
                    tracing::debug!(?note_commitment, ?note, "found note while scanning");
                    Some((*note_commitment, note))
                } else {
                    None
                }
            },
        )
        .collect();

    // Notes we've found for us in this block
    let new_notes: Vec<NoteRecord>;

    if decrypted_notes.is_empty() {
        // We didn't find any notes for us in this block
        new_notes = Vec::new();

        // If there are no notes we care about in this block, just insert the block root into the
        // tree instead of processing each commitment individually
        note_commitment_tree
            .insert_block(block_root)
            .expect("inserting a block root must succeed");
    } else {
        // If we found at least one note for us in this block, we have to explicitly construct the
        // whole block in the NCT by inserting each commitment one at a time
        new_notes = note_payloads
            .iter()
            .filter_map(|note_payload| {
                let note_commitment = note_payload.note_commitment;

                if let Some(note) = decrypted_notes.remove(&note_commitment) {
                    // Keep track of this commitment for later witnessing
                    note_commitment_tree
                        .insert(tct::Witness::Keep, note_commitment)
                        .expect("inserting a commitment must succeed");

                    let position = note_commitment_tree
                        .position_of(note_commitment)
                        .expect("witnessed note commitment is present");

                    let nullifier = fvk.derive_nullifier(position, &note_commitment);

                    let diversifier = &note.diversifier();

                    let record = NoteRecord {
                        note_commitment,
                        height_spent: None,
                        height_created: height,
                        note,
                        diversifier_index: fvk.incoming().index_for_diversifier(diversifier),
                        nullifier,
                        position,
                    };

                    Some(record)
                } else {
                    // Don't remember this commitment; it wasn't ours
                    note_commitment_tree
                        .insert(tct::Witness::Forget, note_commitment)
                        .expect("inserting a commitment must succeed");

                    None
                }
            })
            .collect();

        // End the block in the commitment tree
        note_commitment_tree
            .end_block()
            .expect("ending the block must succed");
    }

    // If we've also reached the end of the epoch, end the epoch in the commitment tree
    if Epoch::from_height(height, epoch_duration).is_epoch_end(height) {
        tracing::debug!(?height, "end of epoch");
        note_commitment_tree
            .end_epoch()
            .expect("ending the epoch must succeed");
    }

    // Print the TCT root for debugging
    tracing::debug!(tct_root = %note_commitment_tree.root(), "tct root");

    // TODO: write a query to mark all matching rows as spent

    ScanResult {
        new_notes,
        spent_nullifiers: nullifiers,
        height,
    }
}
