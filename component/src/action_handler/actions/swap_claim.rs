use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_chain::StateReadExt as _;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{action::SwapClaim, Transaction};
use tracing::instrument;

use crate::{
    action_handler::ActionHandler, dex::StateReadExt as _, shielded_pool::StateReadExt as _,
};

#[async_trait]
impl ActionHandler for SwapClaim {
    #[instrument(name = "swap_claim", skip(self))]
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        let swap_claim = self;

        let fee = swap_claim.body.fee.clone();

        // Check swap claim proof
        let anchor = context.anchor;
        swap_claim
            .proof
            .verify(
                anchor,
                swap_claim.body.nullifier,
                swap_claim.body.output_data,
                swap_claim.body.epoch_duration,
                swap_claim.body.output_1.note_commitment,
                swap_claim.body.output_2.note_commitment,
                fee,
                swap_claim.body.output_1.ephemeral_key,
                swap_claim.body.output_2.ephemeral_key,
            )
            .context("a swap claim proof did not verify")?;

        // TODO: any other stateless checks?

        Ok(())
    }

    #[instrument(name = "swap_claim", skip(self, state, context))]
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        let swap_claim = self;

        // 1. Validate the epoch duration passed in the swap claim matches
        // what we know.
        let epoch_duration = state.get_epoch_duration().await?;
        let provided_epoch_duration = swap_claim.body.epoch_duration;
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
        state.check_nullifier_unspent(spent_nullifier).await;

        Ok(())
    }

    #[instrument(name = "swap_claim", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        // Nothing to do here, note payloads and nullifiers processed in shielded pool

        Ok(())
    }
}
