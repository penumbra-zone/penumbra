use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::{component::StateReadExt as _, NoteSource, SpendInfo};
use penumbra_crypto::{Address, Note, NotePayload, Nullifier, Rseed, Value};
use penumbra_proto::StateWriteProto;
use penumbra_sct::component::{SctManager as _, StateReadExt as _};
use penumbra_storage::StateWrite;
use penumbra_tct as tct;
use tct::StateCommitment;
use tracing::instrument;

use crate::{event, state_key};

use super::SupplyWrite;

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
        source: NoteSource,
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
            .state_commitment_tree()
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

        let note = Note::from_parts(*address, value, Rseed(rseed_bytes))?;
        // Now record the note and update the total supply:
        self.update_token_supply(&value.asset_id, value.amount.value() as i128)
            .await?;
        self.add_note_payload(note.payload(), source).await;

        Ok(())
    }

    #[instrument(skip(self, note_payload, source), fields(commitment = ?note_payload.note_commitment))]
    async fn add_note_payload(&mut self, note_payload: NotePayload, source: NoteSource) {
        tracing::debug!(source = ?source);

        // 0. Record an ABCI event for transaction indexing.
        //self.record(event::state_payload(&payload));

        // 1. Insert it into the SCT, recording its note source:
        let position = self.add_sct_commitment(note_payload.note_commitment, Some(source))
            .await
            // TODO: why? can't we exceed the number of state commitments in a block?
            .expect("inserting into the state commitment tree should not fail because we should budget commitments per block (currently unimplemented)");

        // 2. Finally, record it to be inserted into the compact block:
        let mut payloads: im::Vector<(tct::Position, NotePayload, NoteSource)> = self
            .object_get(state_key::pending_notes())
            .unwrap_or_default();
        payloads.push_back((position, note_payload, source));
        self.object_put(state_key::pending_notes(), payloads);
    }

    #[instrument(skip(self, note_commitment))]
    async fn add_rolled_up_payload(&mut self, note_commitment: StateCommitment) {
        tracing::debug!(?note_commitment);

        // 0. Record an ABCI event for transaction indexing.
        //self.record(event::state_payload(&payload));

        // 1. Insert it into the SCT:
        let position = self.add_sct_commitment(note_commitment, None)
            .await
            // TODO: why? can't we exceed the number of state commitments in a block?
            .expect("inserting into the state commitment tree should not fail because we should budget commitments per block (currently unimplemented)");

        // 2. Finally, record it to be inserted into the compact block:
        let mut payloads: im::Vector<(tct::Position, StateCommitment)> = self
            .object_get(state_key::pending_rolled_up_notes())
            .unwrap_or_default();
        payloads.push_back((position, note_commitment));
        self.object_put(state_key::pending_rolled_up_notes(), payloads);
    }

    async fn pending_note_payloads(&self) -> im::Vector<(tct::Position, NotePayload, NoteSource)> {
        self.object_get(state_key::pending_notes())
            .unwrap_or_default()
    }

    async fn pending_rolled_up_payloads(&self) -> im::Vector<(tct::Position, StateCommitment)> {
        self.object_get(state_key::pending_rolled_up_notes())
            .unwrap_or_default()
    }

    #[instrument(skip(self, source))]
    async fn spend_nullifier(&mut self, nullifier: Nullifier, source: NoteSource) {
        tracing::debug!("marking as spent");

        // We need to record the nullifier as spent in the JMT (to prevent
        // double spends), as well as in the CompactBlock (so that clients
        // can learn that their note was spent).
        self.put(
            state_key::spent_nullifier_lookup(&nullifier),
            // We don't use the value for validity checks, but writing the source
            // here lets us find out what transaction spent the nullifier.
            SpendInfo {
                note_source: source,
                spend_height: self.get_block_height().await.expect("block height is set"),
            },
        );
        // Also record an ABCI event for transaction indexing.
        self.record(event::spend(&nullifier));

        // Record the nullifier to be inserted into the compact block
        let mut nullifiers: im::Vector<Nullifier> = self
            .object_get(state_key::pending_nullifiers())
            .unwrap_or_default();
        nullifiers.push_back(nullifier);
        self.object_put(state_key::pending_nullifiers(), nullifiers);
    }
}

impl<T: StateWrite + ?Sized> NoteManager for T {}
