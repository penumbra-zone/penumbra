use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::{component::StateReadExt as _, NoteSource, SpendInfo};
use penumbra_crypto::{Address, Note, NotePayload, Nullifier, Rseed, Value};
use penumbra_proto::StateWriteProto;
use penumbra_sct::component::{SctManager as _, StateReadExt as _};
use penumbra_storage::StateWrite;
use tracing::instrument;

use crate::{event, state_key};

use super::SupplyWrite;

#[derive(Clone)]
pub struct StatePayload {
    pub source: NoteSource,
    pub note: NotePayload,
}

pub struct StatePayloadDebugKind<'a>(pub &'a StatePayload);

impl<'a> std::fmt::Debug for StatePayloadDebugKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Note").finish_non_exhaustive()
    }
}

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
        self.add_state_payload(StatePayload {
            note: note.payload(),
            source,
        })
        .await;

        Ok(())
    }

    #[instrument(skip(self, payload), fields(commitment = ?payload.note.note_commitment))]
    async fn add_state_payload(&mut self, payload: StatePayload) {
        tracing::debug!(payload = ?StatePayloadDebugKind(&payload));

        // 0. Record an ABCI event for transaction indexing.
        //self.record(event::state_payload(&payload));

        // 1. Insert it into the SCT
        self.add_sct_commitment(payload.note.note_commitment)
            .await
            // TODO: why? can't we exceed the number of state commitments in a block?
            .expect("inserting into the state commitment tree should not fail because we should budget commitments per block (currently unimplemented)");

        // 2. Record its source in the JMT
        self.put(
            state_key::note_source(&payload.note.note_commitment),
            payload.source,
        );

        // 3. Finally, record it to be inserted into the compact block:
        let mut payloads: im::Vector<StatePayload> = self
            .object_get(state_key::pending_payloads())
            .unwrap_or_default();
        payloads.push_back(payload);
        self.object_put(state_key::pending_payloads(), payloads);
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
