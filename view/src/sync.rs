use std::collections::{BTreeMap, BTreeSet};

use penumbra_chain::{
    params::FmdParameters, AnnotatedNotePayload, CompactBlock, Epoch, NoteSource,
};
use penumbra_crypto::{FullViewingKey, IdentityKey, Note, NotePayload, Nullifier, PayloadKey};
use penumbra_tct as tct;

use crate::{QuarantinedNoteRecord, SpendableNoteRecord, Storage};

/// Contains the results of scanning a single block.
#[derive(Debug, Clone)]
pub struct FilteredBlock {
    pub new_notes: Vec<SpendableNoteRecord>,
    pub new_quarantined_notes: Vec<QuarantinedNoteRecord>,
    pub spent_nullifiers: Vec<Nullifier>,
    pub spent_quarantined_nullifiers: BTreeMap<IdentityKey, Vec<Nullifier>>,
    pub slashed_validators: Vec<IdentityKey>,
    pub height: u64,
    pub fmd_parameters: Option<FmdParameters>,
}

impl FilteredBlock {
    pub fn all_nullifiers(&self) -> impl Iterator<Item = &Nullifier> {
        self.spent_quarantined_nullifiers
            .values()
            .flat_map(|v| v.iter())
            .chain(self.spent_nullifiers.iter())
    }

    pub fn inbound_transaction_ids(&self) -> BTreeSet<[u8; 32]> {
        let mut ids = BTreeSet::new();
        let sources = self.new_notes.iter().map(|n| n.source);
        //.chain(self.new_quarantined_notes.iter().map(|n| n.source));
        for source in sources {
            if let NoteSource::Transaction { id } = source {
                ids.insert(id);
            }
        }
        ids
    }
}

#[tracing::instrument(skip(fvk, note_commitment_tree, note_payloads, nullifiers, storage))]
pub async fn scan_block(
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
        fmd_parameters,
        proposal_started,
    }: CompactBlock,
    epoch_duration: u64,
    storage: &Storage,
) -> anyhow::Result<FilteredBlock> {
    // Trial-decrypt a note with our own specific viewing key
    let trial_decrypt =
        |note_payload: NotePayload| -> tokio::task::JoinHandle<Option<(Note, PayloadKey)>> {
            // TODO: change fvk to Arc<FVK> in Worker and pass to scan_block as Arc
            // need this so the task is 'static and not dependent on key lifetime
            let fvk2 = fvk.clone();
            tokio::spawn(async move { note_payload.trial_decrypt(&fvk2) })
        };

    //The NotePayload contains the EphemeralKey, which can be used with the IVK from the FVK to derive the PayloadKey

    // Notes we've found in this block that are meant for us
    let new_notes: Vec<SpendableNoteRecord>;
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
            spent_quarantined_nullifiers
                .entry(identity_key)
                .or_default()
                .extend(unbonding.nullifiers);
            // Trial-decrypt the quarantined notes, keeping track of the ones that were meant for us
            let decryptions = unbonding
                .note_payloads
                .into_iter()
                .map(|AnnotatedNotePayload { payload, source }| (trial_decrypt(payload), source))
                .collect::<Vec<_>>();
            for (decryption, source) in decryptions {
                if let Some((note, payload_key)) = decryption.await.unwrap() {
                    new_quarantined_notes.push(QuarantinedNoteRecord {
                        note_commitment: note.commit(),
                        height_created: height,
                        address_index: fvk.incoming().index_for_diversifier(note.diversifier()),
                        note,
                        unbonding_epoch,
                        identity_key,
                        source,
                        payload_key,
                    });
                }
            }
        }
    }

    // Trial-decrypt the notes in this block, keeping track of the ones that were meant for us
    let decryptions = note_payloads
        .iter()
        .map(|annotated| trial_decrypt(annotated.payload.clone()))
        .collect::<Vec<_>>();
    let mut decrypted_applied_notes = BTreeMap::new();
    for decryption in decryptions {
        if let Some((note, key)) = decryption.await.unwrap() {
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
        new_notes = note_payloads
            .into_iter()
            .filter_map(|AnnotatedNotePayload { payload, source }| {
                let note_commitment = payload.note_commitment;

                if let Some(note) = decrypted_applied_notes.remove(&note_commitment) {
                    // Keep track of this commitment for later witnessing
                    let position = note_commitment_tree
                        .insert(tct::Witness::Keep, note_commitment)
                        .expect("inserting a commitment must succeed");

                    let nullifier = fvk.derive_nullifier(position, &note_commitment);

                    let diversifier = note.diversifier();
                    let address_index = fvk.incoming().index_for_diversifier(diversifier);
                    let epk = &payload.ephemeral_key;
                    let shared_secret = fvk.incoming().key_agreement_with(epk).ok()?;
                    let payload_key = PayloadKey::derive(&shared_secret, epk);
                    let record = SpendableNoteRecord {
                        note_commitment,
                        height_spent: None,
                        height_created: height,
                        note,
                        address_index,
                        nullifier,
                        position,
                        source,
                        payload_key,
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

    //Filter nullifiers to remove any without matching note commitments

    let filtered_nullifiers = storage.filter_nullifiers(spent_nullifiers).await?;

    let mut filtered_quarantined_nullifiers = BTreeMap::new();

    for (id, nullifiers) in spent_quarantined_nullifiers {
        filtered_quarantined_nullifiers.insert(id, storage.filter_nullifiers(nullifiers).await?);
    }

    // Construct filtered block

    let result = FilteredBlock {
        new_notes,
        new_quarantined_notes,
        spent_nullifiers: filtered_nullifiers,
        spent_quarantined_nullifiers: filtered_quarantined_nullifiers,
        slashed_validators: slashed,
        height,
        fmd_parameters,
    };

    if !result.spent_quarantined_nullifiers.is_empty() || !result.new_quarantined_notes.is_empty() {
        tracing::debug!(?result, "scan result contained quarantined things");
    }

    Ok(result)
}
