use std::sync::Arc;

use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sdk_proof_params::SWAP_PROOF_VERIFICATION_KEY;
use penumbra_sdk_proto::{DomainType as _, StateWriteProto};
use penumbra_sdk_sct::component::source::SourceContext;

use crate::{
    component::{InternalDexWrite, StateReadExt, SwapDataWrite, SwapManager},
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
                balance_commitment: self.balance_commitment_inner(),
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

        // Accumulate the swap's flows, crediting the DEX VCB for the inflows.
        let flow = (swap.body.delta_1_i, swap.body.delta_2_i);
        state
            .accumulate_swap_flow(&swap.body.trading_pair, flow.into())
            .await?;

        // Record the swap commitment in the state.
        let source = state.get_current_source().expect("source is set");
        state
            .add_swap_payload(self.body.payload.clone(), source.into())
            .await;

        // Mark the assets for the swap's trading pair as accessed during this block.
        let fixed_candidates = Arc::new(dex_params.fixed_candidates.clone());
        state.add_recently_accessed_asset(
            swap.body.trading_pair.asset_1(),
            fixed_candidates.clone(),
        );
        state.add_recently_accessed_asset(swap.body.trading_pair.asset_2(), fixed_candidates);

        state.record_proto(event::EventSwap::from(self).to_proto());

        Ok(())
    }
}
