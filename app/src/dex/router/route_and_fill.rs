use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_crypto::{
    asset,
    dex::{BatchSwapOutputData, TradingPair},
    Amount, SwapFlow, Value,
};
use penumbra_storage::StateWrite;

use crate::dex::{
    router::{FillRoute, PathSearch},
    PositionManager, StateWriteExt,
};

/// Ties together the routing and filling logic, to process
/// a block's batch swap flows.
#[async_trait]
pub trait RouteAndFill: StateWrite + Sized {
    async fn handle_batch_swaps(
        self: &mut Arc<Self>,
        trading_pair: TradingPair,
        batch_data: SwapFlow,
        block_height: u64,
        epoch_height: u64,
    ) -> Result<()>
    where
        Self: 'static,
    {
        let (delta_1, delta_2) = (batch_data.0.mock_decrypt(), batch_data.1.mock_decrypt());

        tracing::debug!(?delta_1, ?delta_2, ?trading_pair);

        // Depending on the contents of the batch swap inputs, we might need to path search in either direction.
        let (lambda_2, unfilled_1) = if delta_1.value() > 0 {
            // There is input for asset 1, so we need to route for asset 1 -> asset 2
            self.route_and_fill_inner(trading_pair.asset_1(), trading_pair.asset_2(), delta_1)
                .await?
        } else {
            // There was no input for asset 1, so there's 0 output for asset 2 from this side.
            (0u64.into(), delta_1)
        };

        let (lambda_1, unfilled_2) = if delta_2.value() > 0 {
            // There is input for asset 2, so we need to route for asset 2 -> asset 1
            self.route_and_fill_inner(trading_pair.asset_2(), trading_pair.asset_1(), delta_2)
                .await?
        } else {
            // There was no input for asset 2, so there's 0 output for asset 1 from this side.
            (0u64.into(), delta_2)
        };

        let output_data = BatchSwapOutputData {
            height: block_height,
            epoch_height,
            trading_pair,
            delta_1,
            delta_2,
            lambda_1_2: lambda_1,
            lambda_2_1: lambda_2,
            lambda_1_1: unfilled_1,
            lambda_2_2: unfilled_2,
        };
        tracing::debug!(?output_data);
        Arc::get_mut(self)
            .expect("expected state to have no other refs")
            .set_output_data(output_data);
        Ok(())
    }

    // TODO: this is publically exposed rn, but should be private
    async fn route_and_fill_inner(
        self: &mut Arc<Self>,
        asset_1: asset::Id,
        asset_2: asset::Id,
        delta_1: Amount,
    ) -> Result<(Amount, Amount)>
    where
        Self: 'static,
    {
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
                // TODO: max hops should not be hardcoded
                .path_search(asset_1, asset_2, 4)
                .await
                .unwrap();

            tracing::debug!("path is some? {}", path.is_some());
            if path.is_none() {
                // No path found, so we can't fill any more.
                break;
            }

            let path = path.unwrap();
            tracing::debug!(?path);

            (outer_lambda_2, outer_unfilled_1) = {
                // path found, fill as much as we can
                let delta_1 = Value {
                    amount: outer_unfilled_1,
                    asset_id: asset_1,
                };
                let (unfilled_1, lambda_2) = Arc::get_mut(self)
                    .expect("expected state to have no other refs")
                    .fill_route(delta_1, &path, spill_price)
                    .await?;
                assert_eq!(lambda_2.asset_id, asset_2);
                assert_eq!(unfilled_1.asset_id, asset_1);
                let lambda_2 = lambda_2.amount;
                tracing::debug!(?lambda_2, ?unfilled_1);

                // The output of asset 2 is the sum of all the `lambda_2` values,
                // and the unfilled amount becomes the new `delta_1`.
                (outer_lambda_2 + lambda_2, unfilled_1.amount)
            };

            if outer_unfilled_1.value() == 0 {
                // All of the `delta_1` was spent
                break;
            }
        }

        Ok((outer_lambda_2, outer_unfilled_1))
    }
}

impl<T: PositionManager> RouteAndFill for T {}
