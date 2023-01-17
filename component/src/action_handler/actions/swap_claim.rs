use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_chain::{sync::StatePayload, StateReadExt as _};
use penumbra_storage::{State, StateRead, StateTransaction, StateWrite};
use penumbra_transaction::{action::SwapClaim, Transaction};
use tracing::instrument;

use crate::{
    action_handler::ActionHandler,
    stubdex::StateReadExt as _,
    shielded_pool::{self, NoteManager, StateReadExt as _},
};

#[async_trait]
impl ActionHandler for SwapClaim {
    #[instrument(name = "swap_claim", skip(self))]
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        self.proof
            .verify(
                context.anchor,
                self.body.nullifier,
                self.body.output_data,
                self.epoch_duration,
                self.body.output_1_commitment,
                self.body.output_2_commitment,
                self.body.fee.clone(),
            )
            .context("a swap claim proof did not verify")?;

        Ok(())
    }

    #[instrument(name = "swap_claim", skip(self, state))]
    async fn check_stateful(&self, state: Arc<State>) -> Result<()> {
        let swap_claim = self;

        // 1. Validate the epoch duration passed in the swap claim matches
        // what we know.
        let epoch_duration = state.get_epoch_duration().await?;
        let provided_epoch_duration = swap_claim.epoch_duration;
        if epoch_duration != provided_epoch_duration {
            return Err(anyhow::anyhow!(
                "provided epoch duration does not match chain epoch duration"
            ));
        }

        // 2. The stateful check *must* validate that the clearing
        // prices used in the proof are valid.
        let provided_output_height = swap_claim.body.output_data.height;
        let provided_trading_pair = swap_claim.body.output_data.trading_pair;
        let output_data = state
            .output_data(provided_output_height, provided_trading_pair)
            .await?
            // This check also ensures that the height for the swap is in the past, otherwise
            // the output data would not be present in the JMT.
            .ok_or_else(|| anyhow::anyhow!("output data not found"))?;

        if output_data != swap_claim.body.output_data {
            return Err(anyhow::anyhow!(
                "provided output data does not match chain output data"
            ));
        }

        // 3. Check that the nullifier hasn't been spent before.
        let spent_nullifier = self.body.nullifier;
        state.check_nullifier_unspent(spent_nullifier).await?;

        Ok(())
    }

    #[instrument(name = "swap_claim", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        // Record the output notes in the state.
        let source = state.object_get("source").cloned().unwrap_or_default();

        state
            .add_state_payload(StatePayload::RolledUp(
                self.body.output_1_commitment.clone(),
            ))
            .await;
        state
            .add_state_payload(StatePayload::RolledUp(
                self.body.output_2_commitment.clone(),
            ))
            .await;

        state.spend_nullifier(self.body.nullifier, source).await;
        // TODO: why do we manage event emission separately up at the top level
        // instead of integrated into state machine?
        state.record(shielded_pool::event::spend(self.body.nullifier));

        Ok(())
    }
}
