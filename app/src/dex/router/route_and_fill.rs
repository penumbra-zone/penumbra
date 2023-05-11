use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_crypto::{
    asset,
    dex::{execution::SwapExecution, BatchSwapOutputData, TradingPair},
    Amount, SwapFlow, Value,
};
use penumbra_storage::StateWrite;
use tracing::instrument;

use crate::dex::{
    router::{FillRoute, PathSearch, RoutingParams},
    PositionManager, StateWriteExt,
};

/// Ties together the routing and filling logic, to process
/// a block's batch swap flows.
#[async_trait]
pub trait HandleBatchSwaps: StateWrite + Sized {
    #[instrument(skip(self, trading_pair, batch_data, block_height, epoch_height, params))]
    async fn handle_batch_swaps(
        self: &mut Arc<Self>,
        trading_pair: TradingPair,
        batch_data: SwapFlow,
        // TODO: why not read these 2 from the state?
        block_height: u64,
        epoch_height: u64,
        params: RoutingParams,
    ) -> Result<()>
    where
        Self: 'static,
    {
        let (delta_1, delta_2) = (batch_data.0.mock_decrypt(), batch_data.1.mock_decrypt());

        tracing::debug!(?delta_1, ?delta_2, ?trading_pair);

        // Since we store a single swap execution struct for the canonical trading pair,
        // representing swaps in both directions, let's set that up now:
        let traces: im::Vector<Vec<Value>> = im::Vector::new();
        Arc::get_mut(self)
            .expect("one mutable reference to state")
            .object_put("trade_traces", traces);

        // Depending on the contents of the batch swap inputs, we might need to path search in either direction.
        let (lambda_2, unfilled_1) = if delta_1.value() > 0 {
            // There is input for asset 1, so we need to route for asset 1 -> asset 2
            self.route_and_fill(
                trading_pair.asset_1(),
                trading_pair.asset_2(),
                delta_1,
                params.clone(),
            )
            .await?
        } else {
            // There was no input for asset 1, so there's 0 output for asset 2 from this side.
            tracing::debug!("no input for asset 1, skipping 1=>2 execution");
            (0u64.into(), delta_1)
        };

        let (lambda_1, unfilled_2) = if delta_2.value() > 0 {
            // There is input for asset 2, so we need to route for asset 2 -> asset 1
            self.route_and_fill(
                trading_pair.asset_2(),
                trading_pair.asset_1(),
                delta_2,
                params.clone(),
            )
            .await?
        } else {
            // There was no input for asset 2, so there's 0 output for asset 1 from this side.
            tracing::debug!("no input for asset 2, skipping 2=>1 execution");
            (0u64.into(), delta_2)
        };

        let output_data = BatchSwapOutputData {
            height: block_height,
            epoch_height,
            trading_pair,
            delta_1,
            delta_2,
            lambda_1,
            lambda_2,
            unfilled_1,
            unfilled_2,
        };

        // TODO: how does this work when there are trades in both directions?
        // Won't that mix up the traces? Should the SwapExecution be indexed by
        // the _DirectedTradingPair_?

        // Fetch the swap execution object that should have been modified during the routing and filling.
        let trade_traces: im::Vector<Vec<Value>> = self
            .object_get("trade_traces")
            .ok_or_else(|| anyhow::anyhow!("missing swap execution in object store2"))?;
        tracing::debug!(?output_data, ?trade_traces);
        Arc::get_mut(self)
            .expect("expected state to have no other refs")
            .set_output_data(
                output_data,
                SwapExecution {
                    traces: trade_traces.into_iter().collect(),
                },
            );

        // Clean up the swap execution object store now that it's been persisted.
        Arc::get_mut(self)
            .expect("expected state to have no other refs")
            .object_delete("trade_traces");

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
    ) -> Result<(Amount, Amount)>
    where
        Self: 'static,
    {
        tracing::debug!(?delta_1, ?asset_1, ?asset_2, "starting route_and_fill");
        // Output of asset 2
        let mut outer_lambda_2 = 0u64.into();
        // Unfilled output of asset 1
        let mut outer_unfilled_1 = delta_1;

        // Continuously route and fill until either:
        // 1. We have no more delta_1 remaining
        // 2. A path can no longer be found
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

            (outer_lambda_2, outer_unfilled_1) = {
                // path found, fill as much as we can
                let delta_1 = Value {
                    amount: outer_unfilled_1,
                    asset_id: asset_1,
                };

                tracing::debug!(?path, delta_1 = ?delta_1.amount, "found path, starting to fill up to spill price");

                // TODO: in what circumstances should we use fill_route_exact?
                let (unfilled_1, lambda_2) = Arc::get_mut(self)
                    .expect("expected state to have no other refs")
                    .fill_route(delta_1, &path, spill_price)
                    .await
                    .context("error filling along best path")?;

                tracing::debug!(lambda_2 = ?lambda_2.amount, unfilled_1 = ?unfilled_1.amount, "filled along best path");

                assert_eq!(lambda_2.asset_id, asset_2);
                assert_eq!(unfilled_1.asset_id, asset_1);

                // The output of asset 2 is the sum of all the `lambda_2` values,
                // and the unfilled amount becomes the new `delta_1`.
                (outer_lambda_2 + lambda_2.amount, unfilled_1.amount)
            };

            if outer_unfilled_1.value() == 0 {
                tracing::debug!("filled all of delta_1, exiting route_and_fill");
                break;
            }
        }

        Ok((outer_lambda_2, outer_unfilled_1))
    }
}

impl<T: HandleBatchSwaps> RouteAndFill for T {}
