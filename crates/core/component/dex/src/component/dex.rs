use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::Component;
use penumbra_asset::{asset, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_num::Amount;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_sct::component::clock::EpochRead;
use tendermint::v0_37::abci;
use tracing::instrument;

use crate::{
    component::flow::SwapFlow, event, genesis, state_key, BatchSwapOutputData, DexParameters,
    DirectedTradingPair, SwapExecution, TradingPair,
};

use super::{
    router::{HandleBatchSwaps, RoutingParams},
    Arbitrage, PositionManager, ValueCircuitBreaker,
};

pub struct Dex {}

#[async_trait]
impl Component for Dex {
    type AppState = genesis::Content;

    #[instrument(name = "dex", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            None => {
                // Checkpoint -- no-op
            }
            Some(app_state) => {
                state.put_dex_params(app_state.dex_params.clone());
            }
        }
    }

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
        // 1. Add all newly opened positions to the DEX.
        // This has already happened in the action handlers for each `PositionOpen` action.

        // 2. For each batch swap during the block, calculate clearing prices and set in the JMT.

        let current_epoch = state.get_current_epoch().await.expect("epoch is set");
        let routing_params = state.routing_params().await.expect("dex params are set");

        for (trading_pair, swap_flows) in state.swap_flows() {
            let batch_start = std::time::Instant::now();
            state
                .handle_batch_swaps(
                    trading_pair,
                    swap_flows,
                    end_block
                        .height
                        .try_into()
                        .expect("height is part of the end block data"),
                    current_epoch.start_height,
                    // Always include both ends of the target pair as fixed candidates.
                    routing_params
                        .clone()
                        .with_extra_candidates([trading_pair.asset_1(), trading_pair.asset_2()]),
                )
                .await
                .expect("handling batch swaps is infaillible");
            metrics::histogram!(crate::component::metrics::DEX_BATCH_DURATION)
                .record(batch_start.elapsed());
        }

        // 3. Perform arbitrage to ensure all prices are consistent post-execution:

        // For arbitrage, we extend the path search by 2 hops to allow a path out of the
        // staking token and back.

        // TODO: Build an extended candidate set with:
        // - both ends of all trading pairs for which there were swaps in the block
        // - both ends of all trading pairs for which positions were opened
        let arb_routing_params = RoutingParams {
            max_hops: routing_params.max_hops + 2,
            fixed_candidates: routing_params.fixed_candidates.clone(),
            price_limit: Some(1u64.into()),
        };

        let arb_burn = match state
            .arbitrage(*STAKING_TOKEN_ASSET_ID, arb_routing_params)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                // The arbitrage search should not error, but if it does, we should
                // simply not perform arbitrage, rather than halting the entire chain.
                tracing::warn!(?e, "error processing arbitrage, this is a bug");
                Value {
                    amount: Amount::zero(),
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                }
            }
        };

        if arb_burn.amount != 0u64.into() {
            // TODO: hack to avoid needing an asset cache for nice debug output
            let unit = asset::Cache::with_known_assets()
                .get_unit("penumbra")
                .expect("penumbra is a known asset");
            let burn = format!("{}{}", unit.format_value(arb_burn.amount), unit);
            // TODO: this should be an ABCI event
            tracing::info!(%burn, "executed arbitrage opportunity");
        }

        // 4. Close all positions queued for closure at the end of the block.
        // It's important to do this after execution, to allow block-scoped JIT liquidity.
        Arc::get_mut(state)
            .expect("state should be uniquely referenced after batch swaps complete")
            .close_queued_positions()
            .await
            .expect("closing queued positions should not fail");
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

    /// Get the swap flow for the given trading pair accumulated in this block so far.
    fn swap_flow(&self, pair: &TradingPair) -> SwapFlow {
        self.swap_flows().get(pair).cloned().unwrap_or_default()
    }

    fn swap_flows(&self) -> BTreeMap<TradingPair, SwapFlow> {
        self.object_get::<BTreeMap<TradingPair, SwapFlow>>(state_key::swap_flows())
            .unwrap_or_default()
    }

    fn pending_batch_swap_outputs(&self) -> im::OrdMap<TradingPair, BatchSwapOutputData> {
        self.object_get(state_key::pending_outputs())
            .unwrap_or_default()
    }

    /// Gets the DEX parameters from the state.
    async fn get_dex_params(&self) -> Result<DexParameters> {
        self.get(state_key::config::dex_params())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Missing DexParameters"))
    }

    /// Indicates if the DEX parameters have been updated in this block.
    fn dex_params_updated(&self) -> bool {
        self.object_get::<()>(state_key::config::dex_params_updated())
            .is_some()
    }

    /// Uses the DEX parameters to construct a `RoutingParams` for use in execution or simulation.
    async fn routing_params(&self) -> Result<RoutingParams> {
        let dex_params = self.get_dex_params().await?;
        Ok(RoutingParams {
            max_hops: dex_params.max_hops as usize,
            fixed_candidates: Arc::new(dex_params.fixed_candidates),
            price_limit: None,
        })
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

/// Extension trait providing write access to dex data.
#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    fn put_dex_params(&mut self, params: DexParameters) {
        self.put(state_key::config::dex_params().to_string(), params);
        self.object_put(state_key::config::dex_params_updated(), ())
    }

    async fn set_output_data(
        &mut self,
        output_data: BatchSwapOutputData,
        swap_execution_1_for_2: Option<SwapExecution>,
        swap_execution_2_for_1: Option<SwapExecution>,
    ) -> Result<()> {
        // Debit the DEX for the swap outflows.
        // Note that since we credited the DEX for _all_ inflows, we need to debit the
        // unfilled amounts as well as the filled amounts.
        //
        // In the case of a value inflation bug, the debit call will return an underflow
        // error, which will halt the chain.
        self.vcb_debit(Value {
            amount: output_data.unfilled_1 + output_data.lambda_1,
            asset_id: output_data.trading_pair.asset_1,
        })
        .await?;
        self.vcb_debit(Value {
            amount: output_data.unfilled_2 + output_data.lambda_2,
            asset_id: output_data.trading_pair.asset_2,
        })
        .await?;

        // Write the output data to the state under a known key, for querying, ...
        let height = output_data.height;
        let trading_pair = output_data.trading_pair;
        self.put(state_key::output_data(height, trading_pair), output_data);

        // Store the swap executions for both directions in the state as well.
        if let Some(swap_execution) = swap_execution_1_for_2.clone() {
            let tp_1_for_2 = DirectedTradingPair::new(trading_pair.asset_1, trading_pair.asset_2);
            self.nonverifiable_put(
                state_key::swap_execution(height, tp_1_for_2)
                    .as_bytes()
                    .to_vec(),
                swap_execution,
            );
        }
        if let Some(swap_execution) = swap_execution_2_for_1.clone() {
            let tp_2_for_1 = DirectedTradingPair::new(trading_pair.asset_2, trading_pair.asset_1);
            self.put(
                state_key::swap_execution(height, tp_2_for_1),
                swap_execution,
            );
        }

        // ... and also add it to the set in the compact block to be pushed out to clients.
        let mut outputs = self.pending_batch_swap_outputs();
        outputs.insert(trading_pair, output_data);
        self.object_put(state_key::pending_outputs(), outputs);

        // Also generate an ABCI event for indexing:
        self.record_proto(event::batch_swap(
            output_data,
            swap_execution_1_for_2,
            swap_execution_2_for_1,
        ));

        Ok(())
    }

    fn set_arb_execution(&mut self, height: u64, execution: SwapExecution) {
        self.put(state_key::arb_execution(height), execution);
    }

    async fn put_swap_flow(
        &mut self,
        trading_pair: &TradingPair,
        swap_flow: SwapFlow,
    ) -> Result<()> {
        // Credit the DEX for the swap inflows.
        //
        // Note that we credit the DEX for _all_ inflows, since we don't know
        // how much will eventually be filled.
        self.vcb_credit(Value {
            amount: swap_flow.0,
            asset_id: trading_pair.asset_1,
        })
        .await?;
        self.vcb_credit(Value {
            amount: swap_flow.1,
            asset_id: trading_pair.asset_2,
        })
        .await?;

        // TODO: replace with IM struct later
        let mut swap_flows = self.swap_flows();
        swap_flows.insert(*trading_pair, swap_flow);
        self.object_put(state_key::swap_flows(), swap_flows);

        Ok(())
    }
}

impl<T: StateWrite> StateWriteExt for T {}
