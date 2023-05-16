use crate::dex::state_key;
use async_trait::async_trait;
use penumbra_chain::NoteSource;
use penumbra_crypto::dex::swap::SwapPayload;
use penumbra_sct::component::SctManager as _;
use penumbra_storage::StateWrite;
use penumbra_tct as tct;
use tracing::instrument;

/// Manages the addition of new notes to the chain state.
#[async_trait]
pub trait SwapManager: StateWrite {
    #[instrument(skip(self, swap), fields(commitment = ?swap.commitment))]
    async fn add_swap_payload(&mut self, swap: SwapPayload, source: NoteSource) {
        tracing::debug!("adding swap payload");

        // 0. Record an ABCI event for transaction indexing.
        //self.record(event::state_payload(&payload));

        // 1. Insert it into the SCT, recording its source
        let position = self.add_sct_commitment(swap.commitment, Some(source))
            .await
            // TODO: why? can't we exceed the number of state commitments in a block?
            .expect("inserting into the state commitment tree should not fail because we should budget commitments per block (currently unimplemented)");

        // 3. Finally, record it to be inserted into the compact block:
        let mut payloads: im::Vector<(tct::Position, SwapPayload, NoteSource)> = self
            .object_get(state_key::pending_payloads())
            .unwrap_or_default();
        payloads.push_back((position, swap, source));
        self.object_put(state_key::pending_payloads(), payloads);
    }

    async fn pending_swap_payloads(&self) -> im::Vector<(tct::Position, SwapPayload, NoteSource)> {
        self.object_get(state_key::pending_payloads())
            .unwrap_or_default()
    }
}

impl<T: StateWrite + ?Sized> SwapManager for T {}
