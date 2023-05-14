use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::{component::StateReadExt as _, NoteSource, SpendInfo};
use penumbra_compact_block::{
    component::{StateReadExt as _, StateWriteExt as _},
    StatePayload, StatePayloadDebugKind,
};
use penumbra_crypto::{Address, Note, Nullifier, Rseed, Value};
use penumbra_proto::StateWriteProto;
use penumbra_sct::component::{StateReadExt as _, StateWriteExt as _};
use penumbra_storage::StateWrite;
use penumbra_tct as tct;
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
            .stub_state_commitment_tree()
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
        self.add_state_payload(StatePayload::Note {
            note: note.payload(),
            source,
        })
        .await;

        Ok(())
    }

    #[instrument(skip(self, payload), fields(commitment = ?payload.commitment()))]
    async fn add_state_payload(&mut self, payload: StatePayload) {
        tracing::debug!(payload = ?StatePayloadDebugKind(&payload));

        // 0. Record an ABCI event for transaction indexing.
        //self.record(event::state_payload(&payload));

        // 1. Insert it into the SCT
        let mut sct = self.stub_state_commitment_tree().await;
        sct.insert(tct::Witness::Forget, *payload.commitment())
            // TODO: why? can't we exceed the number of state commitments in a block?
            .expect("inserting into the state commitment tree never fails");
        self.stub_put_state_commitment_tree(&sct);

        // 2. Record its source in the JMT, if present
        if let Some(source) = payload.source() {
            self.put(state_key::note_source(payload.commitment()), *source);
        }

        // 3. Finally, record it in the pending compact block.
        let mut compact_block = self.stub_compact_block();
        compact_block.state_payloads.push(payload);
        self.stub_put_compact_block(compact_block);
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

        let mut compact_block = self.stub_compact_block();
        compact_block.nullifiers.push(nullifier);
        self.stub_put_compact_block(compact_block);
    }
}

impl<T: StateWrite + ?Sized> NoteManager for T {}
