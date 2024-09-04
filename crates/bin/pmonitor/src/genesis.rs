use std::collections::BTreeMap;

use penumbra_compact_block::{CompactBlock, StatePayload};
use penumbra_keys::FullViewingKey;
use penumbra_shielded_pool::{Note, NotePayload};
use penumbra_tct::StateCommitment;

use tracing::Instrument;

#[derive(Debug, Clone)]
pub struct FilteredGenesisBlock {
    // Store new notes per FVK
    //
    // TODO: Make this store UM-equivalent balance per FVK.
    pub notes: BTreeMap<String, BTreeMap<StateCommitment, Note>>,
}

/// Scanning of the genesis `CompactBlock` with a list of FVKs to determine the
/// initial balances of the relevant addresses.
///
/// Assumption: There are no swaps or nullifiers in the genesis block.
#[tracing::instrument(skip_all, fields(height = %height))]
pub async fn scan_genesis_block(
    fvks: Vec<FullViewingKey>,
    CompactBlock {
        height,
        state_payloads,
        ..
    }: CompactBlock,
) -> anyhow::Result<FilteredGenesisBlock> {
    assert_eq!(height, 0);

    let mut genesis_notes = BTreeMap::new();

    // We proceed one FVK at a time.
    for fvk in fvks {
        // Trial-decrypt a note with our a specific viewing key
        let trial_decrypt_note =
            |note_payload: NotePayload| -> tokio::task::JoinHandle<Option<Note>> {
                let fvk2 = fvk.clone();
                tokio::spawn(
                    async move { note_payload.trial_decrypt(&fvk2) }
                        .instrument(tracing::Span::current()),
                )
            };

        // Trial-decrypt the notes in this block, keeping track of the ones that were meant for the FVK
        // we're monitoring.
        let mut note_decryptions = Vec::new();

        // We only care about notes, so we're ignoring swaps and rolled-up commitments.
        for payload in state_payloads.iter() {
            if let StatePayload::Note { note, .. } = payload {
                note_decryptions.push(trial_decrypt_note((**note).clone()));
            }
        }

        let mut notes_for_this_fvk = BTreeMap::new();
        for decryption in note_decryptions {
            if let Some(note) = decryption
                .await
                .expect("able to join tokio note decryption handle")
            {
                notes_for_this_fvk.insert(note.commit(), note);
            }
        }

        // Save all the notes for this FVK, and continue.
        genesis_notes.insert(fvk.to_string(), notes_for_this_fvk);
    }

    // Construct filtered genesis block with allocations
    let result = FilteredGenesisBlock {
        notes: genesis_notes,
    };

    Ok(result)
}
