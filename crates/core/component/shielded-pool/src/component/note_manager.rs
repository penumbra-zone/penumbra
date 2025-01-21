use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sdk_asset::Value;
use penumbra_sdk_keys::Address;
use penumbra_sdk_sct::component::tree::{SctManager, SctRead};
use penumbra_sdk_sct::CommitmentSource;
use penumbra_sdk_tct as tct;
use tct::StateCommitment;
use tracing::instrument;

use crate::state_key;
use crate::{Note, NotePayload, Rseed};

/// Manages the addition of new notes to the chain state.
#[async_trait]
pub trait NoteManager: StateWrite {
    /// Mint a new (public) note into the shielded pool.
    ///
    /// Most notes in the shielded pool are created by client transactions.
    /// This method allows the chain to inject new value into the shielded pool
    /// on its own.
    #[instrument(skip(self, value, address, source))]
    async fn mint_note(
        &mut self,
        value: Value,
        address: &Address,
        source: CommitmentSource,
    ) -> Result<()> {
        tracing::debug!(?value, ?address, "minting tokens");

        // These notes are public, so we don't need a blinding factor for
        // privacy, but since the note commitments are determined by the note
        // contents, we need to have unique (deterministic) blinding factors for
        // each note, so they cannot collide.
        //
        // Hashing the current SCT root would be sufficient, since it will
        // change every time we insert a new note.  But computing the SCT root
        // is very slow, so instead we hash the current position.
        let position: u64 = self
            .get_sct()
            .await
            .position()
            .expect("state commitment tree is not full")
            .into();

        let rseed_bytes: [u8; 32] = blake2b_simd::Params::default()
            .personal(b"PenumbraMint")
            .to_state()
            .update(&position.to_le_bytes())
            .finalize()
            .as_bytes()[0..32]
            .try_into()?;

        let note = Note::from_parts(address.clone(), value, Rseed(rseed_bytes))?;
        self.add_note_payload(note.payload(), source).await;

        Ok(())
    }

    #[instrument(skip(self, note_payload, source), fields(commitment = ?note_payload.note_commitment))]
    async fn add_note_payload(&mut self, note_payload: NotePayload, source: CommitmentSource) {
        tracing::debug!(source = ?source);

        // 0. Record an ABCI event for transaction indexing.
        //self.record(event::state_payload(&payload));

        // 1. Insert it into the SCT, recording its note source:
        let position = self.add_sct_commitment(note_payload.note_commitment, source.clone())
            .await
            // TODO: why? can't we exceed the number of state commitments in a block?
            .expect("inserting into the state commitment tree should not fail because we should budget commitments per block (currently unimplemented)");

        // 2. Finally, record it to be inserted into the compact block:
        let mut payloads = self.pending_note_payloads();
        payloads.push_back((position, note_payload, source));
        self.object_put(state_key::pending_notes(), payloads);
    }

    #[instrument(skip(self, note_commitment))]
    async fn add_rolled_up_payload(
        &mut self,
        note_commitment: StateCommitment,
        source: CommitmentSource,
    ) {
        tracing::debug!(?note_commitment);

        // 0. Record an ABCI event for transaction indexing.
        //self.record(event::state_payload(&payload));

        // 1. Insert it into the SCT:
        let position = self.add_sct_commitment(note_commitment, source)
            .await
            // TODO: why? can't we exceed the number of state commitments in a block?
            .expect("inserting into the state commitment tree should not fail because we should budget commitments per block (currently unimplemented)");

        // 2. Finally, record it to be inserted into the compact block:
        let mut payloads = self.pending_rolled_up_payloads();
        payloads.push_back((position, note_commitment));
        self.object_put(state_key::pending_rolled_up_payloads(), payloads);
    }

    fn pending_note_payloads(&self) -> im::Vector<(tct::Position, NotePayload, CommitmentSource)> {
        self.object_get(state_key::pending_notes())
            .unwrap_or_default()
    }

    fn pending_rolled_up_payloads(&self) -> im::Vector<(tct::Position, StateCommitment)> {
        self.object_get(state_key::pending_rolled_up_payloads())
            .unwrap_or_default()
    }
}

impl<T: StateWrite + ?Sized> NoteManager for T {}
