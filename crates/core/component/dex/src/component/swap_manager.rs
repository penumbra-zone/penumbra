use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_asset::Value;
use penumbra_sdk_sct::{component::tree::SctManager, CommitmentSource};
use penumbra_sdk_tct as tct;
use tracing::instrument;

use crate::component::circuit_breaker::value::ValueCircuitBreaker;
use crate::BatchSwapOutputData;
use crate::SwapExecution;
use crate::{
    component::flow::SwapFlow, state_key, swap::SwapPayload, DirectedTradingPair, TradingPair,
};
use anyhow::Result;
use penumbra_sdk_proto::StateWriteProto;

/// Manages the addition of new notes to the chain state.
#[async_trait]
pub(crate) trait SwapManager: StateWrite {
    #[instrument(skip(self, swap), fields(commitment = ?swap.commitment))]
    async fn add_swap_payload(&mut self, swap: SwapPayload, source: CommitmentSource) {
        tracing::trace!("adding swap payload");

        // Record the swap commitment and its metadata in the SCT
        let position = self.add_sct_commitment(swap.commitment, source.clone())
            .await
            // TODO(erwan): Tracked in #830: we should handle this gracefully
            .expect("inserting into the state commitment tree should not fail because we should budget commitments per block (currently unimplemented)");

        // Record the payload in object-storage so that we can include it in this block's [`CompactBlock`].
        let mut payloads = self.pending_swap_payloads();
        payloads.push_back((position, swap, source));
        self.object_put(state_key::pending_payloads(), payloads);
    }
}

impl<T: StateWrite + ?Sized> SwapManager for T {}

pub trait SwapDataRead: StateRead {
    fn pending_swap_payloads(&self) -> im::Vector<(tct::Position, SwapPayload, CommitmentSource)> {
        self.object_get(state_key::pending_payloads())
            .unwrap_or_default()
    }

    /// Get the swap flow for the given trading pair accumulated in this block so far.
    fn swap_flow(&self, pair: &TradingPair) -> SwapFlow {
        self.swap_flows().get(pair).cloned().unwrap_or_default()
    }

    fn swap_flows(&self) -> im::OrdMap<TradingPair, SwapFlow> {
        self.object_get::<im::OrdMap<TradingPair, SwapFlow>>(state_key::swap_flows())
            .unwrap_or_default()
    }

    fn pending_batch_swap_outputs(&self) -> im::OrdMap<TradingPair, BatchSwapOutputData> {
        self.object_get(state_key::pending_outputs())
            .unwrap_or_default()
    }
}

impl<T: StateRead + ?Sized> SwapDataRead for T {}

pub(crate) trait SwapDataWrite: StateWrite {
    async fn accumulate_swap_flow(
        &mut self,
        trading_pair: &TradingPair,
        swap_flow: SwapFlow,
    ) -> Result<()> {
        // Credit the DEX for the swap inflows.
        //
        // At this point we don't know how much will eventually be filled, so we
        // credit for all inflows, and then later debit for any unfilled input
        // in the BSOD.
        self.dex_vcb_credit(Value {
            amount: swap_flow.0,
            asset_id: trading_pair.asset_1,
        })
        .await?;
        self.dex_vcb_credit(Value {
            amount: swap_flow.1,
            asset_id: trading_pair.asset_2,
        })
        .await?;

        // Accumulate the new swap flow into the map.
        let old = self.swap_flows();
        let new = old.alter(
            |maybe_flow| match maybe_flow {
                Some(flow) => Some((flow.0 + swap_flow.0, flow.1 + swap_flow.1).into()),
                None => Some(swap_flow),
            },
            *trading_pair,
        );
        self.object_put(state_key::swap_flows(), new);

        Ok(())
    }

    fn put_swap_execution_at_height(
        &mut self,
        height: u64,
        pair: DirectedTradingPair,
        swap_execution: SwapExecution,
    ) {
        let path = state_key::swap_execution(height, pair);
        self.nonverifiable_put(path.as_bytes().to_vec(), swap_execution);
    }
}

impl<T: StateWrite + ?Sized> SwapDataWrite for T {}
