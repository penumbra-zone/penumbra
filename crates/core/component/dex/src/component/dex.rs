use std::collections::BTreeSet;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::Component;
use penumbra_asset::asset;
use penumbra_asset::{Value, STAKING_TOKEN_ASSET_ID};
use penumbra_proto::{StateReadProto, StateWriteProto};
use tendermint::v0_37::abci;
use tracing::instrument;

use crate::state_key::block_scoped;
use crate::{
    component::SwapDataRead, component::SwapDataWrite, event, genesis, state_key,
    BatchSwapOutputData, DexParameters, DirectedTradingPair, SwapExecution, TradingPair,
};

use super::eviction_manager::EvictionManager;
use super::{
    chandelier::Chandelier,
    router::{HandleBatchSwaps, RoutingParams},
    Arbitrage, PositionManager, PositionRead as _, ValueCircuitBreaker,
};

pub struct Dex {}

#[async_trait]
impl Component for Dex {
    type AppState = genesis::Content;

    #[instrument(name = "dex", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            None => { /* no-op */ }
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
        let routing_params = state.routing_params().await.expect("dex params are set");
        let execution_budget = state
            .get_dex_params()
            .await
            .expect("dex params are set")
            .max_execution_budget;

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
                    // Always include both ends of the target pair as fixed candidates.
                    routing_params
                        .clone()
                        .with_extra_candidates([trading_pair.asset_1(), trading_pair.asset_2()]),
                    execution_budget,
                )
                .await
                .expect("handling batch swaps is infaillible");
            metrics::histogram!(crate::component::metrics::DEX_BATCH_DURATION)
                .record(batch_start.elapsed());
        }

        // 3. Perform arbitrage to ensure all prices are consistent post-execution:

        // For arbitrage, we extend the path search by 2 hops to allow a path out of the
        // staking token and back.

        // Extend the fixed candidate set to include recently accessed assets, to have
        // more arbitrage execution against newly opened positions.
        let fixed_candidates = Arc::new(
            routing_params
                .fixed_candidates
                .iter()
                .cloned()
                // The set of recently accessed assets is already limited to avoid
                // potentially blowing up routing time.
                .chain(state.recently_accessed_assets().iter().cloned())
                .collect::<Vec<_>>(),
        );

        let arb_routing_params = RoutingParams {
            max_hops: routing_params.max_hops + 2,
            fixed_candidates,
            price_limit: Some(1u64.into()),
        };

        match state
            .arbitrage(*STAKING_TOKEN_ASSET_ID, arb_routing_params)
            .await
        {
            // The arb search completed successfully, and surfaced some surplus.
            Ok(Some(v)) => tracing::info!(surplus = ?v, "arbitrage successful!"),
            // The arb completed without errors, but resulted in no surplus, so
            // the state fork was discarded.
            Ok(None) => tracing::debug!("no arbitrage found"),
            // The arbitrage search should not error, but if it does, we should
            // simply not perform arbitrage, rather than halting the entire chain.
            Err(e) => tracing::warn!(?e, "error processing arb, this is a bug"),
        }

        // 4. Inspect trading pairs that saw new position opened during this block, and
        // evict their excess LPs if any are found.
        let _ = Arc::get_mut(state)
            .expect("state should be uniquely referenced after batch swaps complete")
            .evict_positions()
            .await
            .map_err(|e| tracing::error!(?e, "error evicting positions, skipping"));

        // 5. Close all positions queued for closure at the end of the block.
        // It's important to do this after execution, to allow block-scoped JIT liquidity.
        Arc::get_mut(state)
            .expect("state should be uniquely referenced after batch swaps complete")
            .close_queued_positions()
            .await
            .expect("closing queued positions should not fail");

        // 5. Finalize the candlestick data for the block.
        Arc::get_mut(state)
            .expect("state should be uniquely referenced after batch swaps complete")
            .finalize_block_candlesticks()
            .await
            .expect("finalizing block candlesticks should not fail");
    }

    #[instrument(name = "dex", skip(_state))]
    async fn end_epoch<S: StateWrite + 'static>(mut _state: &mut Arc<S>) -> Result<()> {
        Ok(())
    }
}

/// Provides public read access to DEX data.
#[async_trait]
pub trait StateReadExt: StateRead {
    /// Gets the DEX parameters from the state.
    async fn get_dex_params(&self) -> Result<DexParameters> {
        self.get(state_key::config::dex_params())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Missing DexParameters"))
    }

    /// Uses the DEX parameters to construct a `RoutingParams` for use in execution or simulation.
    async fn routing_params(&self) -> Result<RoutingParams> {
        self.get_dex_params().await.map(RoutingParams::from)
    }

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
        self.nonverifiable_get(state_key::swap_execution(height, trading_pair).as_bytes())
            .await
    }

    async fn arb_execution(&self, height: u64) -> Result<Option<SwapExecution>> {
        self.get(&state_key::arb_execution(height)).await
    }

    /// Return a set of [`TradingPair`]s for which liquidity positions were opened
    /// during this block.
    fn get_active_trading_pairs_in_block(&self) -> BTreeSet<TradingPair> {
        self.object_get(block_scoped::active::trading_pairs())
            .unwrap_or_default()
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

/// Extension trait providing write access to dex data.
#[async_trait]
pub trait StateWriteExt: StateWrite {
    fn put_dex_params(&mut self, params: DexParameters) {
        self.put(state_key::config::dex_params().to_string(), params);
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

/// The maximum number of "hot" asset identifiers to track for this block.
const RECENTLY_ACCESSED_ASSET_LIMIT: usize = 10;

/// Provide write access to internal dex data.
pub(crate) trait InternalDexWrite: StateWrite {
    /// Adds an asset ID to the list of recently accessed assets,
    /// making it a candidate for the current block's arbitrage routing.
    ///
    /// This ensures that assets associated with recently active positions
    /// will be eligible for arbitrage if mispriced positions are opened.
    #[tracing::instrument(level = "debug", skip_all)]
    fn add_recently_accessed_asset(
        &mut self,
        asset_id: asset::Id,
        fixed_candidates: Arc<Vec<asset::Id>>,
    ) {
        let mut assets = self.recently_accessed_assets();

        // Limit the number of recently accessed assets to prevent blowing
        // up routing time.
        if assets.len() >= RECENTLY_ACCESSED_ASSET_LIMIT {
            return;
        }

        // If the asset is already in the fixed candidate list, don't insert it.
        if fixed_candidates.contains(&asset_id) {
            return;
        }

        assets.insert(asset_id);
        self.object_put(state_key::recently_accessed_assets(), assets);
    }

    /// Mark a [`TradingPair`] as active during this block.
    fn mark_trading_pair_as_active(&mut self, pair: TradingPair) {
        let mut active_pairs = self.get_active_trading_pairs_in_block();

        if active_pairs.insert(pair) {
            self.object_put(block_scoped::active::trading_pairs(), active_pairs)
        }
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
        self.dex_vcb_debit(Value {
            amount: output_data.unfilled_1 + output_data.lambda_1,
            asset_id: output_data.trading_pair.asset_1,
        })
        .await?;
        self.dex_vcb_debit(Value {
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
            self.put_swap_execution_at_height(height, tp_1_for_2, swap_execution);
        }
        if let Some(swap_execution) = swap_execution_2_for_1.clone() {
            let tp_2_for_1 = DirectedTradingPair::new(trading_pair.asset_2, trading_pair.asset_1);
            self.put_swap_execution_at_height(height, tp_2_for_1, swap_execution);
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
}

impl<T: StateWrite + ?Sized> InternalDexWrite for T {}
