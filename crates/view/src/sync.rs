use std::collections::{BTreeMap, BTreeSet};

use penumbra_chain::{
    params::{ChainParameters, FmdParameters},
    NoteSource,
};
use penumbra_compact_block::{CompactBlock, StatePayload};
use penumbra_crypto::{FullViewingKey, Note, NotePayload, Nullifier};
use penumbra_dex::swap::{SwapPayload, SwapPlaintext};
use penumbra_tct as tct;
use tracing::Instrument;

use crate::{SpendableNoteRecord, Storage, SwapRecord};

/// Contains the results of scanning a single block.
#[derive(Debug, Clone)]
pub struct FilteredBlock {
    pub new_notes: Vec<SpendableNoteRecord>,
    pub new_swaps: Vec<SwapRecord>,
    pub spent_nullifiers: Vec<Nullifier>,
    pub height: u64,
    pub fmd_parameters: Option<FmdParameters>,
    pub chain_parameters: Option<ChainParameters>,
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

#[tracing::instrument(skip_all, fields(height = %height))]
pub async fn scan_block(
    fvk: &FullViewingKey,
    state_commitment_tree: &mut tct::Tree,
    CompactBlock {
        height,
        state_payloads,
        nullifiers,
        block_root,
        epoch_root,
        fmd_parameters,
        swap_outputs,
        chain_parameters,
        // TODO: do we need this, or is there a bug in scan_block?
        // proposal_started,
        ..
    }: CompactBlock,
    storage: &Storage,
) -> anyhow::Result<FilteredBlock> {
    // Trial-decrypt a note with our own specific viewing key
    let trial_decrypt_note = |note_payload: NotePayload| -> tokio::task::JoinHandle<Option<Note>> {
        // TODO: change fvk to Arc<FVK> in Worker and pass to scan_block as Arc
        // need this so the task is 'static and not dependent on key lifetime
        let fvk2 = fvk.clone();
        tokio::spawn(
            async move { note_payload.trial_decrypt(&fvk2) }.instrument(tracing::Span::current()),
        )
    };
    // Trial-decrypt a swap with our own specific viewing key
    let trial_decrypt_swap =
        |swap_payload: SwapPayload| -> tokio::task::JoinHandle<Option<SwapPlaintext>> {
            // TODO: change fvk to Arc<FVK> in Worker and pass to scan_block as Arc
            // need this so the task is 'static and not dependent on key lifetime
            let fvk2 = fvk.clone();
            tokio::spawn(
                async move { swap_payload.trial_decrypt(&fvk2) }
                    .instrument(tracing::Span::current()),
            )
        };

    // Nullifiers we've found in this block
    let spent_nullifiers: Vec<Nullifier> = nullifiers;

    // Trial-decrypt the notes in this block, keeping track of the ones that were meant for us
    let mut note_decryptions = Vec::new();
    let mut swap_decryptions = Vec::new();
    let mut unknown_commitments = Vec::new();

    for payload in state_payloads.iter() {
        match payload {
            StatePayload::Note { note, .. } => {
                note_decryptions.push(trial_decrypt_note((**note).clone()));
            }
            StatePayload::Swap { swap, .. } => {
                swap_decryptions.push(trial_decrypt_swap((**swap).clone()));
            }
            StatePayload::RolledUp(commitment) => unknown_commitments.push(*commitment),
        }
    }
    // Having started trial decryption in the background, ask the Storage for scanning advice:
    let mut note_advice = storage.scan_advice(unknown_commitments).await?;
    for decryption in note_decryptions {
        if let Some(note) = decryption.await.unwrap() {
            note_advice.insert(note.commit(), note);
        }
    }
    let mut swap_advice = BTreeMap::new();
    for decryption in swap_decryptions {
        if let Some(swap) = decryption.await.unwrap() {
            swap_advice.insert(swap.swap_commitment(), swap);
        }
    }

    // Newly detected spendable notes.
    let mut new_notes = Vec::new();
    // Newly detected claimable swaps.
    let mut new_swaps = Vec::new();

    if note_advice.is_empty() && swap_advice.is_empty() {
        // If there are no notes we care about in this block, just insert the block root into the
        // tree instead of processing each commitment individually
        state_commitment_tree
            .insert_block(block_root)
            .expect("inserting a block root must succeed");
    } else {
        // If we found at least one note for us in this block, we have to explicitly construct the
        // whole block in the SCT by inserting each commitment one at a time
        tracing::debug!("found at least one relevant SCT entry, reconstructing block subtree");

        for payload in state_payloads.into_iter() {
            // We need to insert each commitment, so use a match statement to ensure we
            // exhaustively cover all possible cases.
            match (
                note_advice.get(payload.commitment()),
                swap_advice.get(payload.commitment()),
            ) {
                (Some(note), None) => {
                    // Keep track of this commitment for later witnessing
                    let position = state_commitment_tree
                        .insert(tct::Witness::Keep, *payload.commitment())
                        .expect("inserting a commitment must succeed");

                    let source = payload.source().cloned().unwrap_or_default();
                    let nullifier = fvk.derive_nullifier(position, payload.commitment());
                    let address_index = fvk.incoming().index_for_diversifier(note.diversifier());

                    new_notes.push(SpendableNoteRecord {
                        note_commitment: *payload.commitment(),
                        height_spent: None,
                        height_created: height,
                        note: note.clone(),
                        address_index,
                        nullifier,
                        position,
                        source,
                    });
                }
                (None, Some(swap)) => {
                    // Keep track of this commitment for later witnessing
                    let position = state_commitment_tree
                        .insert(tct::Witness::Keep, *payload.commitment())
                        .expect("inserting a commitment must succeed");

                    let Some(output_data) = swap_outputs.get(&swap.trading_pair).cloned() else {
                        // We've been given an invalid compact block, but we
                        // should keep going, because the fullnode we're talking
                        // to could be lying to us and handing us crafted blocks
                        // with garbage data only we can see, in order to
                        // pinpoint whether or not we control a specific address,
                        // so we can't let on that we've noticed any problem.
                        tracing::warn!("invalid compact block, batch swap output data missing for trading pair {:?}", swap.trading_pair);
                        continue;
                    };

                    // Record the output notes for the future swap claim, so we can detect
                    // them when the swap is claimed.
                    let (output_1, output_2) = swap.output_notes(&output_data);
                    storage.give_advice(output_1).await?;
                    storage.give_advice(output_2).await?;

                    let source = payload.source().cloned().unwrap_or_default();
                    let nullifier = fvk.derive_nullifier(position, payload.commitment());

                    new_swaps.push(SwapRecord {
                        swap_commitment: *payload.commitment(),
                        swap: swap.clone(),
                        position,
                        nullifier,
                        source,
                        output_data,
                        height_claimed: None,
                    });
                }
                (None, None) => {
                    // Don't remember this commitment; it wasn't ours
                    state_commitment_tree
                        .insert(tct::Witness::Forget, *payload.commitment())
                        .expect("inserting a commitment must succeed");
                }
                (Some(_), Some(_)) => unreachable!("swap and note commitments are distinct"),
            }
        }

        // End the block in the commitment tree
        state_commitment_tree
            .end_block()
            .expect("ending the block must succed");
    }

    // If we've also reached the end of the epoch, end the epoch in the commitment tree
    let is_epoch_end = epoch_root.is_some();
    if is_epoch_end {
        tracing::debug!(?height, "end of epoch");
        state_commitment_tree
            .end_epoch()
            .expect("ending the epoch must succeed");
    }

    // Print the TCT root for debugging
    #[cfg(feature = "sct-divergence-check")]
    tracing::debug!(tct_root = %state_commitment_tree.root(), "tct root");

    //Filter nullifiers to remove any without matching note commitments

    let filtered_nullifiers = storage.filter_nullifiers(spent_nullifiers).await?;

    // Construct filtered block
    let result = FilteredBlock {
        new_notes,
        new_swaps,
        spent_nullifiers: filtered_nullifiers,
        height,
        fmd_parameters,
        chain_parameters,
    };

    Ok(result)
}
