use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sdk_asset::{asset, Value};
use penumbra_sdk_num::Amount;
use penumbra_sdk_sct::component::clock::EpochRead;
use tracing::instrument;

use crate::{
    component::{
        chandelier::Chandelier,
        flow::SwapFlow,
        router::{FillRoute, PathSearch, RoutingParams},
        ExecutionCircuitBreaker, InternalDexWrite, PositionManager,
    },
    lp::position::MAX_RESERVE_AMOUNT,
    BatchSwapOutputData, SwapExecution, TradingPair,
};

use super::fill_route::FillError;

/// Ties together the routing and filling logic, to process
/// a block's batch swap flows.
#[async_trait]
pub trait HandleBatchSwaps: StateWrite + Sized {
    #[instrument(skip(self, trading_pair, batch_data, block_height, params))]
    async fn handle_batch_swaps(
        self: &mut Arc<Self>,
        trading_pair: TradingPair,
        batch_data: SwapFlow,
        block_height: u64,
        params: RoutingParams,
        execution_budget: u32,
    ) -> Result<BatchSwapOutputData>
    where
        Self: 'static,
    {
        let (delta_1, delta_2) = (batch_data.0, batch_data.1);
        tracing::debug!(?delta_1, ?delta_2, ?trading_pair, "decrypted batch swaps");

        // We initialize a circuit breaker for this batch swap. This will limit the number of frontier
        // executions up to the specified `execution_budget` parameter.
        let execution_circuit_breaker = ExecutionCircuitBreaker::new(execution_budget);

        // We clamp the deltas to the maximum input for batch swaps.
        let clamped_delta_1 = delta_1.min(MAX_RESERVE_AMOUNT.into());
        let clamped_delta_2 = delta_2.min(MAX_RESERVE_AMOUNT.into());

        tracing::debug!(
            ?clamped_delta_1,
            ?clamped_delta_2,
            "clamped deltas to maximum amount"
        );

        let swap_execution_1_for_2 = self
            .route_and_fill(
                trading_pair.asset_1(),
                trading_pair.asset_2(),
                clamped_delta_1,
                params.clone(),
                execution_circuit_breaker.clone(),
            )
            .await?;

        let swap_execution_2_for_1 = self
            .route_and_fill(
                trading_pair.asset_2(),
                trading_pair.asset_1(),
                clamped_delta_2,
                params.clone(),
                execution_circuit_breaker,
            )
            .await?;

        let (lambda_2, unfilled_1) = match &swap_execution_1_for_2 {
            Some(swap_execution) => (
                swap_execution.output.amount,
                // The unfilled amount of asset 1 is the trade input minus the amount consumed, plus the excess.
                delta_1 - swap_execution.input.amount,
            ),
            None => (0u64.into(), delta_1),
        };
        let (lambda_1, unfilled_2) = match &swap_execution_2_for_1 {
            Some(swap_execution) => (
                swap_execution.output.amount,
                delta_2 - swap_execution.input.amount,
            ),
            None => (0u64.into(), delta_2),
        };
        let epoch = self.get_current_epoch().await.expect("epoch is set");
        let output_data = BatchSwapOutputData {
            height: block_height,
            trading_pair,
            delta_1,
            delta_2,
            lambda_1,
            lambda_2,
            unfilled_1,
            unfilled_2,
            sct_position_prefix: (
                u16::try_from(epoch.index).expect("epoch index should be small enough"),
                // The block index is determined by looking at how many blocks have elapsed since
                // the start of the epoch.
                u16::try_from(block_height - epoch.start_height)
                    .expect("block index should be small enough"),
                0,
            )
                .into(),
        };

        tracing::debug!(
            ?output_data,
            ?swap_execution_1_for_2,
            ?swap_execution_2_for_1
        );

        // Update the candlestick tracking
        if let Some(se) = swap_execution_1_for_2.clone() {
            tracing::debug!("updating candlestick for 1=>2 swap");
            Arc::get_mut(self)
                .expect("expected state to have no other refs")
                .record_swap_execution(&se)
                .await;
        }
        if let Some(se) = &swap_execution_2_for_1 {
            tracing::debug!("updating candlestick for 2=>1 swap");
            Arc::get_mut(self)
                .expect("expected state to have no other refs")
                .record_swap_execution(se)
                .await;
        }

        // Fetch the swap execution object that should have been modified during the routing and filling.
        Arc::get_mut(self)
            .expect("expected state to have no other refs")
            .set_output_data(output_data, swap_execution_1_for_2, swap_execution_2_for_1)
            .await?;

        Ok(output_data)
    }
}

impl<T: PositionManager> HandleBatchSwaps for T {}

/// Lower-level trait that ties together the routing and filling logic.
#[async_trait]
pub trait RouteAndFill: StateWrite + Sized {
    #[instrument(skip(self, asset_1, asset_2, input, params, execution_circuit_breaker))]
    async fn route_and_fill(
        self: &mut Arc<Self>,
        asset_1: asset::Id,
        asset_2: asset::Id,
        input: Amount,
        params: RoutingParams,
        mut execution_circuit_breaker: ExecutionCircuitBreaker,
    ) -> Result<Option<SwapExecution>>
    where
        Self: 'static,
    {
        tracing::debug!(?input, ?asset_1, ?asset_2, "prepare to route and fill");

        if input == Amount::zero() {
            tracing::debug!("no input, short-circuit exit");
            return Ok(None);
        }

        // Unfilled output of asset 1
        let mut total_unfilled_1 = input;
        // Output of asset 2
        let mut total_output_2 = 0u64.into();

        // An ordered list of execution traces that were used to fill the trade.
        let mut traces: Vec<Vec<Value>> = Vec::new();

        // Termination conditions:
        // 1. We have no more `delta_1` remaining
        // 2. A path can no longer be found
        // 3. We have reached the `RoutingParams` specified price limit
        // 4. The execution circuit breaker has been triggered based on the number of path searches and executions
        // 5. An unrecoverable error occurred during the execution of the route.
        loop {
            // Check if we have exceeded the execution circuit breaker limits.
            if execution_circuit_breaker.exceeded_limits() {
                tracing::debug!("execution circuit breaker triggered, exiting route_and_fill");
                break;
            } else {
                // This should be done ahead of doing any path search or execution, so that we never
                // have to reason about the specific control flow of our batch swap logic.
                execution_circuit_breaker.increment();
            }

            // Find the best route between the two assets in the trading pair.
            let (path, spill_price) = self
                .path_search(asset_1, asset_2, params.clone())
                .await
                .context("error finding best path")?;

            let Some(path) = path else {
                tracing::debug!("no path found, exiting route_and_fill");
                break;
            };

            if path.is_empty() {
                tracing::debug!("empty path found, exiting route_and_fill");
                break;
            }

            // We prepare the input for this execution round, which is the remaining unfilled amount of asset 1.
            let delta_1 = Value {
                asset_id: asset_1,
                amount: total_unfilled_1,
            };

            tracing::debug!(?path, ?delta_1, "found path, filling up to spill price");

            let execution_result = Arc::get_mut(self)
                .expect("expected state to have no other refs")
                .fill_route(delta_1, &path, spill_price)
                .await;

            let swap_execution = match execution_result {
                Ok(execution) => execution,
                Err(FillError::ExecutionOverflow(position_id)) => {
                    // We have encountered an overflow during the execution of the route.
                    // To route around this, we will close the position and try to route and fill again.
                    tracing::debug!(culprit = ?position_id, "overflow detected during routing execution");
                    Arc::get_mut(self)
                        .expect("expected state to have no other refs")
                        .close_position_by_id(&position_id)
                        .await
                        .expect("the position still exists");
                    continue;
                }
                Err(e) => {
                    // We have encountered an error during the execution of the route,
                    // there are no clear ways to route around this, so we propagate the error.
                    // `fill_route` is transactional and will have rolled back the state.
                    anyhow::bail!("error filling route: {:?}", e);
                }
            };

            // Immediately track the execution in the state.
            (total_output_2, total_unfilled_1) = {
                // The exact amount of asset 1 that was consumed in this execution round.
                let consumed_input = swap_execution.input;
                // The output of this execution round is the amount of asset 2 that was filled.
                let produced_output = swap_execution.output;

                tracing::debug!(consumed_input = ?consumed_input.amount, output = ?produced_output.amount, "filled along best path");

                // Sanity check that the input and output assets are correct.
                assert_eq!(produced_output.asset_id, asset_2);
                assert_eq!(consumed_input.asset_id, asset_1);

                // Append the traces from this execution to the outer traces.
                traces.append(&mut swap_execution.traces.clone());

                (
                    // The total output of asset 2 is the sum of all outputs.
                    total_output_2 + produced_output.amount,
                    // The total unfilled amount of asset 1 is the remaining unfilled amount minus the amount consumed.
                    total_unfilled_1 - consumed_input.amount,
                )
            };

            if total_unfilled_1.value() == 0 {
                tracing::debug!("filled all input, exiting route_and_fill");
                break;
            }

            // Ensure that we've actually executed, or else bail out.
            let Some(accurate_max_price) = swap_execution.max_price() else {
                tracing::debug!("no traces in execution, exiting route_and_fill");
                break;
            };

            // Check that the execution price is below the price limit, if one is set.
            if let Some(price_limit) = params.price_limit {
                if accurate_max_price >= price_limit {
                    tracing::debug!(
                        ?accurate_max_price,
                        ?price_limit,
                        "execution price above price limit, exiting route_and_fill"
                    );
                    break;
                }
            }
        }

        // If we didn't execute against any position at all, there are no execution records to return.
        if traces.is_empty() {
            return Ok(None);
        } else {
            Ok(Some(SwapExecution {
                traces,
                input: Value {
                    asset_id: asset_1,
                    // The total amount of asset 1 that was actually consumed across rounds.
                    amount: input - total_unfilled_1,
                },
                output: Value {
                    asset_id: asset_2,
                    amount: total_output_2,
                },
            }))
        }
    }
}

impl<T: HandleBatchSwaps> RouteAndFill for T {}
