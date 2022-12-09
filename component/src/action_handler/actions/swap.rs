use std::sync::Arc;

use crate::{
    dex::{StateReadExt as _, StateWriteExt as _},
    shielded_pool::NoteManager,
};
use anyhow::{Context, Result};
use ark_ff::Zero;
use async_trait::async_trait;
use decaf377::Fr;
use penumbra_chain::AnnotatedNotePayload;
use penumbra_crypto::{MockFlowCiphertext, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_storage::{State, StateRead, StateTransaction};
use penumbra_transaction::{action::Swap, Transaction};
use tracing::instrument;

use crate::action_handler::ActionHandler;

#[async_trait]
impl ActionHandler for Swap {
    #[instrument(name = "swap", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        let swap = self;

        // Check swap proof
        swap.proof
            .verify(
                // TODO: no value commitments until flow encryption is available
                // so we pass placeholder values here, the proof doesn't check these right now
                // and will fail when checking is re-enabled.
                Value {
                    amount: 0u64.into(),
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                }
                .commit(Fr::zero()),
                Value {
                    amount: 0u64.into(),
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                }
                .commit(Fr::zero()),
                swap.body.fee_commitment,
                swap.body.swap_nft.note_commitment,
                swap.body.swap_nft.ephemeral_key,
            )
            .context("A swap proof did not verify")?;

        // TODO: are any other checks necessary?

        Ok(())
    }

    #[instrument(name = "swap", skip(self, _state))]
    async fn check_stateful(&self, _state: Arc<State>) -> Result<()> {
        // TODO: are any other checks necessary?

        Ok(())
    }

    #[instrument(name = "swap", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        let swap = self;

        // All swaps will be tallied for the block so the
        // BatchSwapOutputData for the trading pair/block height can
        // be set during `end_block`.
        let mut swap_flow = state.swap_flow(&swap.body.trading_pair);

        // Add the amount of each asset being swapped to the batch swap flow.
        swap_flow.0 += MockFlowCiphertext::new(swap.body.delta_1_i.into());
        swap_flow.1 += MockFlowCiphertext::new(swap.body.delta_2_i.into());

        // Set the batch swap flow for the trading pair.
        state.put_swap_flow(&swap.body.trading_pair, swap_flow);

        // Record the Swap NFT in the state.
        let source = state.object_get("source").cloned().unwrap_or_default();
        state
            .add_note(AnnotatedNotePayload {
                source,
                payload: swap.body.swap_nft.clone(),
            })
            .await;

        Ok(())
    }
}
