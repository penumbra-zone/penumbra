use std::sync::Arc;

use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_proof_params::SWAP_PROOF_VERIFICATION_KEY;
use penumbra_proto::StateWriteProto;
use penumbra_sct::component::source::SourceContext;

use crate::{
    component::{position_manager::PositionManager as _, StateReadExt, StateWriteExt, SwapManager},
    event,
    swap::{proof::SwapProofPublic, Swap},
};

#[async_trait]
impl ActionHandler for Swap {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Check that the trading pair is distinct.
        if self.body.trading_pair.asset_1() == self.body.trading_pair.asset_2() {
            anyhow::bail!("Trading pair must be distinct");
        }

        self.proof.verify(
            &SWAP_PROOF_VERIFICATION_KEY,
            SwapProofPublic {
                balance_commitment: self.body.balance_commitment,
                swap_commitment: self.body.payload.commitment,
                fee_commitment: self.body.fee_commitment,
            },
        )?;

        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // Only execute the swap if the dex is enabled in the dex params.
        let dex_params = state.get_dex_params().await?;

        ensure!(
            dex_params.is_enabled,
            "Dex MUST be enabled to process swap actions."
        );

        let swap = self;

        // All swaps will be tallied for the block so the
        // BatchSwapOutputData for the trading pair/block height can
        // be set during `end_block`.
        let mut swap_flow = state.swap_flow(&swap.body.trading_pair);

        // Add the amount of each asset being swapped to the batch swap flow.
        swap_flow.0 += swap.body.delta_1_i;
        swap_flow.1 += swap.body.delta_2_i;

        // Set the batch swap flow for the trading pair.
        state
            .put_swap_flow(&swap.body.trading_pair, swap_flow)
            .await?;

        // Record the swap commitment in the state.
        let source = state.get_current_source().expect("source is set");
        state
            .add_swap_payload(self.body.payload.clone(), source)
            .await;

        // Mark the assets for the swap's trading pair as accessed during this block.
        let fixed_candidates = Arc::new(dex_params.fixed_candidates.clone());
        state.add_recently_accessed_asset(
            swap.body.trading_pair.asset_1(),
            fixed_candidates.clone(),
        );
        state.add_recently_accessed_asset(swap.body.trading_pair.asset_2(), fixed_candidates);

        state.record_proto(event::swap(self));

        Ok(())
    }
}
