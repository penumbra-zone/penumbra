use anyhow::Result;
use ark_ff::PrimeField;
use async_trait::async_trait;
use decaf377::{Fq, Fr};
use penumbra_chain::{sync::AnnotatedNotePayload, NoteSource};
use penumbra_crypto::{ka, Address, Note, NotePayload, Nullifier, One, Value};
use penumbra_storage::StateWrite;
use penumbra_tct as tct;
use tracing::instrument;

use super::{
    component::{StateReadExt, StateWriteExt},
    state_key, SupplyWrite,
};

/// Manages the addition of new notes to the chain state.
#[async_trait]
pub trait NoteManager: StateWrite {
    /// Mint a new (public) note into the shielded pool.
    ///
    /// Most notes in the shielded pool are created by client transactions.
    /// This method allows the chain to inject new value into the shielded pool
    /// on its own.
    #[instrument(
        skip(self, value, address, source),
        // fields(
        //     position = u64::from(self
        //         .note_commitment_tree
        //         .position()
        //         .unwrap_or_else(|| u64::MAX.into())),
        // )
    )]
    async fn mint_note(
        &mut self,
        value: Value,
        address: &Address,
        source: NoteSource,
    ) -> Result<()> {
        tracing::debug!(?value, ?address, "minting tokens");

        // These notes are public, so we don't need a blinding factor for
        // privacy, but since the note commitments are determined by the note
        // contents, we need to have unique (deterministic) blinding factors for
        // each note, so they cannot collide.
        //
        // Hashing the current NCT root would be sufficient, since it will
        // change every time we insert a new note.  But computing the NCT root
        // is very slow, so instead we hash the current position.
        //
        // TODO: how does this work if we were to build the NCT only at the end of the block?

        let position: u64 = self
            .stub_note_commitment_tree()
            .await
            .position()
            .expect("note commitment tree is not full")
            .into();

        let blinding_factor = Fq::from_le_bytes_mod_order(
            blake2b_simd::Params::default()
                .personal(b"PenumbraMint")
                .to_state()
                .update(&position.to_le_bytes())
                .finalize()
                .as_bytes(),
        );

        let note = Note::from_parts(*address, value, blinding_factor)?;
        let note_commitment = note.commit();

        // Scanning assumes that notes are encrypted, so we need to create
        // note ciphertexts, even if the plaintexts are known.  Use the key
        // "1" to ensure we have contributory behaviour in note encryption.
        let esk = ka::Secret::new_from_field(Fr::one());
        let ephemeral_key = esk.diversified_public(&note.diversified_generator());
        let encrypted_note = note.encrypt(&esk);

        // Now record the note and update the total supply:
        self.update_token_supply(&value.asset_id, i64::from(value.amount))
            .await?;
        self.add_note(AnnotatedNotePayload {
            payload: NotePayload {
                note_commitment,
                ephemeral_key,
                encrypted_note,
            },
            source,
        })
        .await;

        Ok(())
    }

    #[instrument(skip(self, ap), fields(note_commitment = ?ap.payload.note_commitment))]
    async fn add_note(&mut self, ap: AnnotatedNotePayload) {
        let AnnotatedNotePayload { payload, source } = ap;
        tracing::debug!("adding note");

        // 1. Insert it into the NCT
        // TODO: build up data incrementally
        let mut nct = self.stub_note_commitment_tree().await;
        nct.insert(tct::Witness::Forget, payload.note_commitment)
            // TODO: why? can't we exceed the number of note commitments in a block?
            .expect("inserting into the note commitment tree never fails");
        self.stub_put_note_commitment_tree(&nct);

        // 2. Record its source in the JMT
        self.put(state_key::note_source(&payload.note_commitment), source);

        // 3. Finally, record it in the pending compact block.
        let mut compact_block = self.stub_compact_block();
        compact_block
            .note_payloads
            .push(AnnotatedNotePayload { payload, source });
        self.stub_put_compact_block(compact_block);
    }

    #[instrument(skip(self, source))]
    async fn spend_nullifier(&mut self, nullifier: Nullifier, source: NoteSource) {
        tracing::debug!("marking as spent");

        // We need to record the nullifier as spent in the JMT (to prevent
        // double spends), as well as in the CompactBlock (so that clients
        // can learn that their note was spent).
        self.put(
            state_key::spent_nullifier_lookup(nullifier),
            // We don't use the value for validity checks, but writing the source
            // here lets us find out what transaction spent the nullifier.
            source,
        );

        let mut compact_block = self.stub_compact_block();
        compact_block.nullifiers.push(nullifier);
        self.stub_put_compact_block(compact_block);
    }
}

impl<T: StateWrite + ?Sized> NoteManager for T {}
