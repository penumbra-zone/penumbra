use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_asset::{asset, Value};
use penumbra_num::Amount;
use penumbra_storage::StateWrite;
use tracing::instrument;

use crate::{
    component::{
        flow::SwapFlow,
        router::{FillRoute, PathSearch, RoutingParams},
        PositionManager, StateWriteExt,
    },
    BatchSwapOutputData, SwapExecution, TradingPair,
};

/// Ties together the routing and filling logic, to process
/// a block's batch swap flows.
#[async_trait]
pub trait HandleBatchSwaps: StateWrite + Sized {
    #[instrument(skip(
        self,
        trading_pair,
        batch_data,
        block_height,
        epoch_starting_height,
        params
    ))]
    async fn handle_batch_swaps(
        self: &mut Arc<Self>,
        trading_pair: TradingPair,
        batch_data: SwapFlow,
        // TODO: why not read these 2 from the state?
        block_height: u64,
        epoch_starting_height: u64,
        params: RoutingParams,
    ) -> Result<()>
    where
        Self: 'static,
    {
        let (delta_1, delta_2) = (batch_data.0, batch_data.1);

        tracing::debug!(?delta_1, ?delta_2, ?trading_pair, "decrypted batch swaps");

        let swap_execution_1_for_2 = if delta_1.value() > 0 {
            Some(
                self.route_and_fill(
                    trading_pair.asset_1(),
                    trading_pair.asset_2(),
                    delta_1,
                    params.clone(),
                )
                .await?,
            )
        } else {
            tracing::debug!("no input for asset 1, skipping 1=>2 routing and execution");
            None
        };

        let swap_execution_2_for_1 = if delta_2.value() > 0 {
            Some(
                self.route_and_fill(
                    trading_pair.asset_2(),
                    trading_pair.asset_1(),
                    delta_2,
                    params.clone(),
                )
                .await?,
            )
        } else {
            tracing::debug!("no input for asset 2, skipping 2=>1 execution");
            None
        };

        let (lambda_2, unfilled_1) = match &swap_execution_1_for_2 {
            Some(swap_execution) => (
                swap_execution.output.amount,
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
        let output_data = BatchSwapOutputData {
            height: block_height,
            epoch_starting_height,
            trading_pair,
            delta_1,
            delta_2,
            lambda_1,
            lambda_2,
            unfilled_1,
            unfilled_2,
        };

        // Fetch the swap execution object that should have been modified during the routing and filling.
        tracing::debug!(
            ?output_data,
            ?swap_execution_1_for_2,
            ?swap_execution_2_for_1
        );
        Arc::get_mut(self)
            .expect("expected state to have no other refs")
            .set_output_data(output_data, swap_execution_1_for_2, swap_execution_2_for_1);

        Ok(())
    }
}

impl<T: PositionManager> HandleBatchSwaps for T {}

/// Lower-level trait that ties together the routing and filling logic.
#[async_trait]
pub trait RouteAndFill: StateWrite + Sized {
    #[instrument(skip(self, asset_1, asset_2, delta_1, params))]
    async fn route_and_fill(
        self: &mut Arc<Self>,
        asset_1: asset::Id,
        asset_2: asset::Id,
        delta_1: Amount,
        params: RoutingParams,
    ) -> Result<SwapExecution>
    where
        Self: 'static,
    {
        tracing::debug!(?delta_1, ?asset_1, ?asset_2, "starting route_and_fill");
        // Output of asset 2
        let mut total_output_2 = 0u64.into();
        // Unfilled output of asset 1
        let mut total_unfilled_1 = delta_1;

        // All traces of trades that were executed.
        let mut traces: Vec<Vec<Value>> = Vec::new();

        // Termination conditions:
        // 1. We have no more delta_1 remaining
        // 2. A path can no longer be found
        // 3. We have reached the `RoutingParams` specified price limit
        loop {
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

            let delta_1 = Value {
                amount: total_unfilled_1,
                asset_id: asset_1,
            };

            tracing::debug!(?path, delta_1 = ?delta_1.amount, "found path, filling up to spill price");

            let execution = Arc::get_mut(self)
                .expect("expected state to have no other refs")
                .fill_route(delta_1, &path, spill_price)
                .await
                .context("error filling along best path")?;

            // Immediately track the execution in the state.
            (total_output_2, total_unfilled_1) = {
                let lambda_2 = execution.output;
                let unfilled_1 = Value {
                    amount: total_unfilled_1
                        .checked_sub(&execution.input.amount)
                        .expect("unable to subtract unfilled input from total input"),
                    asset_id: asset_1,
                };
                tracing::debug!(input = ?delta_1.amount, output = ?lambda_2.amount, unfilled = ?unfilled_1.amount, "filled along best path");

                assert_eq!(lambda_2.asset_id, asset_2);
                assert_eq!(unfilled_1.asset_id, asset_1);

                // Append the traces from this execution to the outer traces.
                traces.append(&mut execution.traces.clone());

                (total_output_2 + lambda_2.amount, unfilled_1.amount)
            };

            if total_unfilled_1.value() == 0 {
                tracing::debug!("filled all input, exiting route_and_fill");
                break;
            }

            // Ensure that we've actually executed, or else bail out.
            let Some(accurate_max_price) = execution.max_price()? else {
                    tracing::debug!("no traces in execution, exiting route_and_fill");
                    break
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

        Ok(SwapExecution {
            traces,
            input: Value {
                asset_id: asset_1,
                amount: delta_1 - total_unfilled_1,
            },
            output: Value {
                asset_id: asset_2,
                amount: total_output_2,
            },
        })
    }
}

impl<T: HandleBatchSwaps> RouteAndFill for T {}
