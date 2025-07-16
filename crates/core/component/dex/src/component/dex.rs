use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::Component;
use penumbra_sdk_asset::asset;
use penumbra_sdk_asset::{Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_fee::component::StateWriteExt as _;
use penumbra_sdk_fee::Fee;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{DomainType as _, StateReadProto, StateWriteProto};
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
        // F.0. Add all non-native fee payments as swap flows.
        let base_fees_and_tips = {
            let state_ref =
                Arc::get_mut(state).expect("should have unique ref at start of Dex::end_block");

            // Extract the accumulated base fees and tips from the fee component, leaving 0 in its place.
            let base_fees_and_tips = state_ref.take_accumulated_base_fees_and_tips();

            // For each nonnative fee asset, add it in as if it were a chain-submitted swap.
            for (asset_id, (base_fee, tip)) in base_fees_and_tips.iter() {
                if *asset_id == *STAKING_TOKEN_ASSET_ID {
                    continue;
                }
                let pair = TradingPair::new(*asset_id, *STAKING_TOKEN_ASSET_ID);
                // We want to swap all of the fees into the native token, the base/tip distinction
                // just affects where the resulting fees go.
                let total = *base_fee + *tip;
                // DANGEROUS: need to be careful about which side of the pair is which,
                // but the existing API is unsafe and fixing it would be a much larger refactor.
                let flow = if pair.asset_1() == *asset_id {
                    (total, Amount::zero())
                } else {
                    (Amount::zero(), total)
                };
                tracing::debug!(
                    ?asset_id,
                    ?base_fee,
                    ?tip,
                    ?total,
                    ?flow,
                    "inserting chain-submitted swap for alt fee token"
                );

                // Accumulate into the swap flows for this block.
                state_ref
                    .accumulate_swap_flow(&pair, flow.into())
                    .await
                    .expect("should be able to credit DEX VCB");
            }

            // Hold on to the list of base fees and tips so we can claim outputs correctly.
            base_fees_and_tips
        };

        // 1. Add all newly opened positions to the DEX.
        // This has already happened in the action handlers for each `PositionOpen` action.

        // 2. For each batch swap during the block, calculate clearing prices and set in the JMT.
        let routing_params = state.routing_params().await.expect("dex params are set");
        let execution_budget = state
            .get_dex_params()
            .await
            .expect("dex params are set")
            .max_execution_budget;

        // Local cache of BSODs used for claiming fee swaps.
        let mut bsods = BTreeMap::new();

        for (trading_pair, swap_flows) in state.swap_flows() {
            let batch_start = std::time::Instant::now();
            let bsod = state
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
                .expect("handling batch swaps is infallible");
            metrics::histogram!(crate::component::metrics::DEX_BATCH_DURATION)
                .record(batch_start.elapsed());

            bsods.insert(trading_pair, bsod);
        }

        // F.1. Having performed all batch swaps, "claim" the base fees and tips.
        // The VCB has already been debited through the BSOD.
        {
            let state_ref =
                Arc::get_mut(state).expect("should have unique ref after finishing batch swaps");
            for (asset_id, (base_fee, tip)) in base_fees_and_tips.iter() {
                if *asset_id == *STAKING_TOKEN_ASSET_ID {
                    // In this case, there was nothing to swap, so there's nothing
                    // to claim and we just accumulate the fee we took back into the fee component.
                    state_ref.raw_accumulate_base_fee(Fee::from_staking_token_amount(*base_fee));
                    state_ref.raw_accumulate_tip(Fee::from_staking_token_amount(*tip));
                    continue;
                }
                let pair = TradingPair::new(*asset_id, *STAKING_TOKEN_ASSET_ID);
                let bsod = bsods
                    .get(&pair)
                    .expect("bsod should be present for chain-submitted swap");

                let (base_input, tip_input) = if pair.asset_1() == *asset_id {
                    ((*base_fee, 0u64.into()), (*tip, 0u64.into()))
                } else {
                    ((0u64.into(), *base_fee), (0u64.into(), *tip))
                };

                let base_output = bsod.pro_rata_outputs(base_input);
                let tip_output = bsod.pro_rata_outputs(tip_input);
                tracing::debug!(
                    ?asset_id,
                    ?base_input,
                    ?tip_input,
                    ?base_output,
                    ?tip_output,
                    "claiming chain-submitted swap for alt fee token"
                );

                // Obtain the base fee and tip amounts in the native token, discarding any unfilled amounts.
                let (swapped_base, swapped_tip) = if pair.asset_1() == *asset_id {
                    // If `asset_id` is `R_1` we want to pull the other leg of the pair.
                    (base_output.1, tip_output.1)
                } else {
                    // and vice-versa. `R_1` contains native tokens.
                    (base_output.0, tip_output.0)
                };

                // Finally, accumulate the swapped base fee and tip back into the fee component.
                // (We already took all the fees out).
                state_ref.raw_accumulate_base_fee(Fee::from_staking_token_amount(swapped_base));
                state_ref.raw_accumulate_tip(Fee::from_staking_token_amount(swapped_tip));
            }
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
        self.record_proto(
            event::EventBatchSwap {
                batch_swap_output_data: output_data,
                swap_execution_1_for_2,
                swap_execution_2_for_1,
            }
            .to_proto(),
        );

        Ok(())
    }

    fn set_arb_execution(&mut self, height: u64, execution: SwapExecution) {
        self.put(state_key::arb_execution(height), execution);
    }
}

impl<T: StateWrite + ?Sized> InternalDexWrite for T {}
