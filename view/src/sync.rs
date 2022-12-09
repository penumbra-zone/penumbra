use std::collections::{BTreeMap, BTreeSet};

use penumbra_chain::{params::FmdParameters, CompactBlock, Epoch, NoteSource, StatePayload};
use penumbra_crypto::{EncryptedNote, FullViewingKey, Note, Nullifier};
use penumbra_tct as tct;

use crate::{SpendableNoteRecord, Storage};

/// Contains the results of scanning a single block.
#[derive(Debug, Clone)]
pub struct FilteredBlock {
    pub new_notes: Vec<SpendableNoteRecord>,
    pub spent_nullifiers: Vec<Nullifier>,
    pub height: u64,
    pub fmd_parameters: Option<FmdParameters>,
}

impl FilteredBlock {
    pub fn inbound_transaction_ids(&self) -> BTreeSet<[u8; 32]> {
        let mut ids = BTreeSet::new();
        let sources = self.new_notes.iter().map(|n| n.source);
        for source in sources {
            if let NoteSource::Transaction { id } = source {
                ids.insert(id);
            }
        }
        ids
    }
}

#[tracing::instrument(skip(fvk, note_commitment_tree, state_payloads, nullifiers, storage))]
pub async fn scan_block(
    fvk: &FullViewingKey,
    note_commitment_tree: &mut tct::Tree,
    CompactBlock {
        height,
        state_payloads,
        nullifiers,
        block_root,
        epoch_root,
        fmd_parameters,
        proposal_started,
        swap_outputs,
    }: CompactBlock,
    epoch_duration: u64,
    storage: &Storage,
) -> anyhow::Result<FilteredBlock> {
    // Trial-decrypt a note with our own specific viewing key
    let trial_decrypt = |note_payload: EncryptedNote| -> tokio::task::JoinHandle<Option<Note>> {
        // TODO: change fvk to Arc<FVK> in Worker and pass to scan_block as Arc
        // need this so the task is 'static and not dependent on key lifetime
        let fvk2 = fvk.clone();
        tokio::spawn(async move { note_payload.trial_decrypt(&fvk2) })
    };

    // Notes we've found in this block that are meant for us
    let new_notes: Vec<SpendableNoteRecord>;

    // Nullifiers we've found in this block
    let spent_nullifiers: Vec<Nullifier> = nullifiers;

    // Trial-decrypt the notes in this block, keeping track of the ones that were meant for us
    let decryptions = state_payloads
        .iter()
        .filter_map(|x| match x {
            StatePayload::Note { note, source } => Some((trial_decrypt(note.clone()), source)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut decrypted_applied_notes = BTreeMap::new();

    for decryption in decryptions {
        if let Some(note) = decryption.0.await.unwrap() {
            decrypted_applied_notes.insert(note.commit(), note);
        }
    }

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
        // TODO: this will desync on rolled up note commitments, rewrite using for loop technology
        new_notes = state_payloads
            .into_iter()
            .filter_map(|x| match x {
                StatePayload::Note { note, source } => {
                    let note_commitment = note.note_commitment;

                    if let Some(note) = decrypted_applied_notes.remove(&note_commitment) {
                        // Keep track of this commitment for later witnessing
                        let position = note_commitment_tree
                            .insert(tct::Witness::Keep, note_commitment)
                            .expect("inserting a commitment must succeed");

                        let nullifier = fvk.derive_nullifier(position, &note_commitment);

                        let diversifier = note.diversifier();
                        let address_index = fvk.incoming().index_for_diversifier(diversifier);

                        let record = SpendableNoteRecord {
                            note_commitment,
                            height_spent: None,
                            height_created: height,
                            note,
                            address_index,
                            nullifier,
                            position,
                            source,
                        };

                        Some(record)
                    } else {
                        // Don't remember this commitment; it wasn't ours
                        note_commitment_tree
                            .insert(tct::Witness::Forget, note_commitment)
                            .expect("inserting a commitment must succeed");

                        None
                    }
                }
                _ => None,
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

    //Filter nullifiers to remove any without matching note commitments

    let filtered_nullifiers = storage.filter_nullifiers(spent_nullifiers).await?;

    // Construct filtered block
    let result = FilteredBlock {
        new_notes,
        spent_nullifiers: filtered_nullifiers,
        height,
        fmd_parameters,
    };

    Ok(result)
}
