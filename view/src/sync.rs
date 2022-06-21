use std::collections::BTreeMap;

use penumbra_chain::{CompactBlock, Epoch};
use penumbra_crypto::{note, IdentityKey, Nullifier};
use penumbra_crypto::{FullViewingKey, Note, NotePayload};
use penumbra_tct as tct;

use crate::{NoteRecord, QuarantinedNoteRecord};

/// Contains the results of scanning a single block.
#[derive(Debug, Clone)]
pub struct ScanResult {
    // write as new rows
    pub new_notes: Vec<NoteRecord>,
    pub new_quarantined_notes: Vec<QuarantinedNoteRecord>,
    // use to update existing rows
    pub spent_nullifiers: Vec<Nullifier>,
    pub spent_quarantined_nullifiers: BTreeMap<IdentityKey, Vec<Nullifier>>,
    pub slashed_validators: Vec<IdentityKey>,
    pub height: u64,
}

impl ScanResult {
    pub fn is_empty(&self) -> bool {
        self.new_notes.is_empty()
            && self.new_quarantined_notes.is_empty()
            && self.spent_nullifiers.is_empty()
            && self.spent_quarantined_nullifiers.is_empty()
            && self.slashed_validators.is_empty()
    }
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
        quarantined,
        slashed,
    }: CompactBlock,
    epoch_duration: u64,
) -> ScanResult {
    // Trial-decrypt a note with our own specific viewing key
    let trial_decrypt = |NotePayload {
                             note_commitment,
                             ephemeral_key,
                             encrypted_note,
                         }: &NotePayload|
     -> Option<Note> {
        // Try to decrypt the encrypted note using the ephemeral key and persistent incoming
        // viewing key -- if it doesn't decrypt, it wasn't meant for us.
        if let Ok(note) = Note::decrypt(encrypted_note.as_ref(), fvk.incoming(), ephemeral_key) {
            tracing::debug!(?note_commitment, ?note, "found note while scanning");
            Some(note)
        } else {
            None
        }
    };

    // Notes we've found in this block that are meant for us
    let new_notes: Vec<NoteRecord>;
    let mut new_quarantined_notes: Vec<QuarantinedNoteRecord> = Vec::new();

    // Nullifiers we've found in this block
    let spent_nullifiers: Vec<Nullifier> = nullifiers;
    let mut spent_quarantined_nullifiers: BTreeMap<IdentityKey, Vec<Nullifier>> = BTreeMap::new();

    // Collect quarantined nullifiers, and add all quarantined notes we can decrypt to the new
    // quarantined notes set
    for (unbonding_epoch, mut scheduled) in quarantined {
        // For any validator slashed in this block, so any quarantined transactions in this block
        // are immediately reverted; we don't even report them to the state, so that the state can
        // avoid worrying about update ordering
        for &identity_key in slashed.iter() {
            scheduled.unschedule_validator(identity_key);
        }

        for (identity_key, unbonding) in scheduled {
            // Remember these nullifiers (not all of them are ours, we have to check the database)
            for nullifier in unbonding.nullifiers {
                spent_quarantined_nullifiers
                    .entry(identity_key)
                    .or_default()
                    .push(nullifier);
            }
            // Trial-decrypt the quarantined notes, keeping track of the ones that were meant for us
            new_quarantined_notes.extend(
                unbonding
                    .note_payloads
                    .into_iter()
                    .filter_map(|note_payload| trial_decrypt(&note_payload))
                    .map(|note| QuarantinedNoteRecord {
                        note_commitment: note.commit(),
                        height_created: height,
                        diversifier_index: fvk
                            .incoming()
                            .index_for_diversifier(&note.diversifier()),
                        note,
                        unbonding_epoch,
                        identity_key,
                    }),
            );
        }
    }

    // Trial-decrypt the notes in this block, keeping track of the ones that were meant for us
    let mut decrypted_applied_notes: BTreeMap<note::Commitment, Note> = note_payloads
        .iter()
        .filter_map(trial_decrypt)
        .map(|note| (note.commit(), note))
        .collect();

    if decrypted_applied_notes.is_empty() {
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

                if let Some(note) = decrypted_applied_notes.remove(&note_commitment) {
                    // Keep track of this commitment for later witnessing
                    let position = note_commitment_tree
                        .insert(tct::Witness::Keep, note_commitment)
                        .expect("inserting a commitment must succeed");

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

    let result = ScanResult {
        new_notes,
        new_quarantined_notes,
        spent_nullifiers,
        spent_quarantined_nullifiers,
        slashed_validators: slashed,
        height,
    };

    if !result.is_empty() {
        tracing::debug!(?result, "scan result");
    }

    result
}
