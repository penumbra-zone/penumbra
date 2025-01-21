use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium_component::ActionHandler;
use penumbra_sdk_txhash::TransactionContext;

use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_proof_params::SWAPCLAIM_PROOF_VERIFICATION_KEY;
use penumbra_sdk_proto::{DomainType as _, StateWriteProto};
use penumbra_sdk_sct::component::{
    source::SourceContext,
    tree::{SctManager, VerificationExt},
    StateReadExt as _,
};
use penumbra_sdk_shielded_pool::component::NoteManager;

use crate::{
    component::StateReadExt,
    event,
    swap_claim::{SwapClaim, SwapClaimProofPublic},
};

#[async_trait]
impl ActionHandler for SwapClaim {
    type CheckStatelessContext = TransactionContext;
    async fn check_stateless(&self, context: TransactionContext) -> Result<()> {
        self.proof
            .verify(
                &SWAPCLAIM_PROOF_VERIFICATION_KEY,
                SwapClaimProofPublic {
                    anchor: context.anchor,
                    nullifier: self.body.nullifier,
                    claim_fee: self.body.fee.clone(),
                    output_data: self.body.output_data,
                    note_commitment_1: self.body.output_1_commitment,
                    note_commitment_2: self.body.output_2_commitment,
                },
            )
            .context("a swap claim proof did not verify")?;

        Ok(())
    }

    async fn check_historical<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        let swap_claim = self;

        // 1. Validate the epoch duration passed in the swap claim matches
        // what we know.
        //
        // SAFETY: this is safe to check here because the epoch duration cannot change during transaction processing.
        let epoch_duration = state.get_epoch_duration_parameter().await?;
        let provided_epoch_duration = swap_claim.epoch_duration;
        if epoch_duration != provided_epoch_duration {
            anyhow::bail!("provided epoch duration does not match chain epoch duration");
        }

        // 2. The stateful check *must* validate that the clearing
        // prices used in the proof are valid.
        //
        // SAFETY: this is safe to check here because the historical batch swap
        // output data will not change.
        let provided_output_height = swap_claim.body.output_data.height;
        let provided_trading_pair = swap_claim.body.output_data.trading_pair;
        let output_data = state
            .output_data(provided_output_height, provided_trading_pair)
            .await?
            // This check also ensures that the height for the swap is in the past, otherwise
            // the output data would not be present in the JMT.
            .ok_or_else(|| anyhow::anyhow!("output data not found"))?;

        if output_data != swap_claim.body.output_data {
            anyhow::bail!("provided output data does not match chain output data");
        }

        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // 3. Check that the nullifier hasn't been spent before.
        let spent_nullifier = self.body.nullifier;
        state.check_nullifier_unspent(spent_nullifier).await?;

        // Record the output notes in the state.
        let source = state
            .get_current_source()
            .expect("source is set during tx execution");

        state
            .add_rolled_up_payload(self.body.output_1_commitment, source.clone())
            .await;
        state
            .add_rolled_up_payload(self.body.output_2_commitment, source.clone())
            .await;

        state.nullify(self.body.nullifier, source).await;

        state.record_proto(event::EventSwapClaim::from(self).to_proto());

        Ok(())
    }
}
