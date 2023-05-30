use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::component::StateReadExt as _;
use penumbra_component::Component;
use penumbra_crypto::{asset, SwapFlow, STAKING_TOKEN_ASSET_ID};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};
use tendermint::v0_34::abci;
use tracing::instrument;

use crate::{state_key, BatchSwapOutputData, DirectedTradingPair, SwapExecution, TradingPair};

use super::{
    router::{HandleBatchSwaps, RoutingParams},
    Arbitrage, PositionManager,
};

pub struct Dex {}

#[async_trait]
impl Component for Dex {
    type AppState = ();

    #[instrument(name = "dex", skip(_state, _app_state))]
    async fn init_chain<S: StateWrite>(_state: S, _app_state: &()) {}

    #[instrument(name = "dex", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "dex", skip(state, end_block))]
    async fn end_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        end_block: &abci::request::EndBlock,
    ) {
        let current_epoch = state.epoch().await.unwrap();

        // For each batch swap during the block, calculate clearing prices and set in the JMT.
        for (trading_pair, swap_flows) in state.swap_flows() {
            state
                .handle_batch_swaps(
                    trading_pair,
                    swap_flows,
                    end_block.height.try_into().expect("missing height"),
                    current_epoch.start_height,
                    // Always include both ends of the target pair as fixed candidates.
                    RoutingParams::default_with_extra_candidates([
                        trading_pair.asset_1(),
                        trading_pair.asset_2(),
                    ]),
                )
                .await
                .expect("unable to process batch swaps");
        }

        // Then, perform arbitrage:
        let arb_burn = state
            .arbitrage(
                *STAKING_TOKEN_ASSET_ID,
                vec![
                    *STAKING_TOKEN_ASSET_ID,
                    asset::REGISTRY.parse_unit("gm").id(),
                    asset::REGISTRY.parse_unit("gn").id(),
                    asset::REGISTRY.parse_unit("test_usd").id(),
                    asset::REGISTRY.parse_unit("test_btc").id(),
                    asset::REGISTRY.parse_unit("test_atom").id(),
                    asset::REGISTRY.parse_unit("test_osmo").id(),
                ],
            )
            .await
            .expect("must be able to process arbitrage");

        if arb_burn.amount != 0u64.into() {
            // TODO: hack to avoid needing an asset cache for nice debug output
            let unit = asset::REGISTRY.parse_unit("penumbra");
            let burn = format!("{}{}", unit.format_value(arb_burn.amount), unit);
            tracing::info!(%burn, "executed arbitrage opportunity");
        }

        // Next, close all positions queued for closure at the end of the block.
        // It's important to do this after execution, to allow block-scoped JIT liquidity.
        Arc::get_mut(state)
            .expect("state should be uniquely referenced after batch swaps complete")
            .close_queued_positions()
            .await
            .unwrap();
    }

    #[instrument(name = "dex", skip(_state))]
    async fn end_epoch<S: StateWrite + 'static>(mut _state: &mut Arc<S>) -> Result<()> {
        Ok(())
    }
}

/// Extension trait providing read access to dex data.
#[async_trait]
pub trait StateReadExt: StateRead {
    async fn output_data(
        &self,
        height: u64,
        trading_pair: TradingPair,
    ) -> Result<Option<BatchSwapOutputData>> {
        self.get(&state_key::output_data(height, trading_pair))
            .await
    }

    async fn swap_execution(
        &self,
        height: u64,
        trading_pair: DirectedTradingPair,
    ) -> Result<Option<SwapExecution>> {
        self.get(&state_key::swap_execution(height, trading_pair))
            .await
    }

    async fn arb_execution(&self, height: u64) -> Result<Option<SwapExecution>> {
        self.get(&state_key::arb_execution(height)).await
    }

    // Get the swap flow for the given trading pair accumulated in this block so far.
    fn swap_flow(&self, pair: &TradingPair) -> SwapFlow {
        self.swap_flows().get(pair).cloned().unwrap_or_default()
    }

    fn swap_flows(&self) -> BTreeMap<TradingPair, SwapFlow> {
        self.object_get::<BTreeMap<TradingPair, SwapFlow>>(state_key::swap_flows())
            .unwrap_or_default()
    }
}

impl<T: StateRead> StateReadExt for T {}

/// Extension trait providing write access to dex data.
#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    fn set_output_data(
        &mut self,
        output_data: BatchSwapOutputData,
        swap_execution_1_for_2: Option<SwapExecution>,
        swap_execution_2_for_1: Option<SwapExecution>,
    ) {
        // Write the output data to the state under a known key, for querying, ...
        let height = output_data.height;
        let trading_pair = output_data.trading_pair;
        self.put(state_key::output_data(height, trading_pair), output_data);

        // Store the swap executions for both directions in the state as well.
        if let Some(swap_execution) = swap_execution_1_for_2 {
            let tp_1_for_2 = DirectedTradingPair::new(trading_pair.asset_1, trading_pair.asset_2);
            self.put(
                state_key::swap_execution(height, tp_1_for_2),
                swap_execution,
            );
        }
        if let Some(swap_execution) = swap_execution_2_for_1 {
            let tp_2_for_1 = DirectedTradingPair::new(trading_pair.asset_2, trading_pair.asset_1);
            self.put(
                state_key::swap_execution(height, tp_2_for_1),
                swap_execution,
            );
        }

        // ... and also add it to the set in the compact block to be pushed out to clients.
        let mut outputs: im::OrdMap<TradingPair, BatchSwapOutputData> = self
            .object_get(state_key::pending_outputs())
            .unwrap_or_default();
        outputs.insert(trading_pair, output_data);
        self.object_put(state_key::pending_outputs(), outputs);
    }

    fn set_arb_execution(&mut self, height: u64, execution: SwapExecution) {
        self.put(state_key::arb_execution(height), execution);
    }

    fn put_swap_flow(&mut self, trading_pair: &TradingPair, swap_flow: SwapFlow) {
        // TODO: replace with IM struct later
        let mut swap_flows = self.swap_flows();
        swap_flows.insert(*trading_pair, swap_flow);
        self.object_put(state_key::swap_flows(), swap_flows)
    }
}

impl<T: StateWrite> StateWriteExt for T {}
