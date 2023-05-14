use crate::dex::state_key;
use async_trait::async_trait;
use penumbra_chain::NoteSource;
use penumbra_crypto::dex::swap::SwapPayload;
use penumbra_sct::component::SctManager as _;
use penumbra_storage::StateWrite;
use penumbra_tct as tct;
use tracing::instrument;

#[derive(Clone)]
pub struct StatePayload {
    pub source: NoteSource,
    pub swap: Box<SwapPayload>,
}

pub struct StatePayloadDebugKind<'a>(pub &'a StatePayload);

impl<'a> std::fmt::Debug for StatePayloadDebugKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Swap").finish_non_exhaustive()
    }
}

/// Manages the addition of new notes to the chain state.
#[async_trait]
pub trait SwapManager: StateWrite {
    #[instrument(skip(self, payload), fields(commitment = ?payload.swap.commitment))]
    async fn add_swap_state_payload(&mut self, payload: StatePayload) {
        tracing::debug!(payload = ?StatePayloadDebugKind(&payload));

        // 0. Record an ABCI event for transaction indexing.
        //self.record(event::state_payload(&payload));

        // 1. Insert it into the SCT, recording its source
        let position = self.add_sct_commitment(payload.swap.commitment, Some(payload.source))
            .await
            // TODO: why? can't we exceed the number of state commitments in a block?
            .expect("inserting into the state commitment tree should not fail because we should budget commitments per block (currently unimplemented)");

        // 3. Finally, record it to be inserted into the compact block:
        let mut payloads: im::Vector<(tct::Position, StatePayload)> = self
            .object_get(state_key::pending_payloads())
            .unwrap_or_default();
        payloads.push_back((position, payload));
        self.object_put(state_key::pending_payloads(), payloads);
    }
}

impl<T: StateWrite + ?Sized> SwapManager for T {}
