use std::sync::Arc;

use crate::{
    shielded_pool::NoteManager,
    stubdex::{StateReadExt as _, StateWriteExt as _},
};
use anyhow::Result;

use async_trait::async_trait;

use penumbra_chain::sync::StatePayload;
use penumbra_crypto::MockFlowCiphertext;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::Swap, IsAction, Transaction};
use tracing::instrument;

use crate::action_handler::ActionHandler;

#[async_trait]
impl ActionHandler for Swap {
    #[instrument(name = "swap", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        self.proof.verify(
            self.body.fee_commitment,
            self.body.payload.commitment,
            self.balance_commitment(),
        )?;

        Ok(())
    }

    #[instrument(name = "swap", skip(self, _state))]
    async fn check_stateful<S: StateRead>(&self, _state: Arc<S>) -> Result<()> {
        Ok(())
    }

    #[instrument(name = "swap", skip(self, state))]
    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
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

        // Record the swap commitment in the state.
        let source = state.object_get("source").unwrap_or_default();
        state
            .add_state_payload(StatePayload::Swap {
                source,
                swap: self.body.payload.clone(),
            })
            .await;

        Ok(())
    }
}
