use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_crypto::{
    asset,
    dex::{lp::position, BatchSwapOutputData, DirectedTradingPair, TradingPair},
    fixpoint::U128x128,
    Amount, SwapFlow, Value,
};
use penumbra_storage::{StateDelta, StateWrite};

use crate::dex::{
    router::{FillRoute, PathSearch},
    PositionManager, PositionRead, StateWriteExt,
};

/// Ties together the routing and filling logic, to process
/// a block's batch swap flows.
#[async_trait]
pub trait RouteAndFill: StateWrite + Sized {
    async fn handle_batch_swaps(
        self: &mut Arc<Self>,
        trading_pair: TradingPair,
        batch_data: SwapFlow,
        // TODO: use price_limit to clamp spill price or set to 1 for arb
        price_limit: Amount,
        block_height: u64,
    ) -> Result<()>
    where
        Self: 'static,
    {
        let (delta_1, delta_2) = (batch_data.0.mock_decrypt(), batch_data.1.mock_decrypt());

        tracing::debug!(?delta_1, ?delta_2, ?trading_pair);

        // Depending on the contents of the batch swap inputs, we might need to path search in either direction.
        if delta_1.value() > 0 {
            // There is input for asset 1, so we need to route for asset 1 -> asset 2
            self.route_and_fill_inner(
                trading_pair.asset_1(),
                trading_pair.asset_2(),
                delta_1,
                delta_2,
            )
            .await?;
        }

        if delta_2.value() > 0 {
            // There is input for asset 2, so we need to route for asset 2 -> asset 1
            self.route_and_fill_inner(
                trading_pair.asset_2(),
                trading_pair.asset_1(),
                delta_2,
                delta_1,
            )
            .await?;
        }

        let output_data = BatchSwapOutputData {
            height: block_height,
            trading_pair,
            delta_1,
            delta_2,
            lambda_1_1,
            lambda_2_2,
            lambda_1_2,
            lambda_2_1,
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
        trading_pair: TradingPair,
        batch_data: SwapFlow,
        // TODO: use price_limit to clamp spill price or set to 1 for arb
        price_limit: Amount,
        block_height: u64,
    ) -> Result<()>
    where
        Self: 'static,
    {
        // Find the best route between the two assets in the trading pair.
        let (path, spill_price) = self
            // TODO: max hops should not be hardcoded
            .path_search(trading_pair.asset_1(), trading_pair.asset_2(), 4)
            .await
            .unwrap();

        tracing::debug!("path is some? {}", path.is_some());

        let (lambda_1, lambda_2, success) = if path.is_some() {
            let path = path.unwrap();
            tracing::debug!(?path);
            // path found, fill as much as we can
            // TODO: what if one of delta_1/delta_2 is zero? don't we need to fill based on the other?
            let delta_1 = Value {
                amount: delta_1,
                asset_id: trading_pair.asset_1(),
            };
            let delta_2 = Value {
                amount: delta_2,
                asset_id: trading_pair.asset_2(),
            };
            let (unfilled_1, lambda_2) = Arc::get_mut(self)
                .expect("expected state to have no other refs")
                .fill_route(delta_1, &path, spill_price.unwrap_or_default())
                .await
                .unwrap();
            let (unfilled_2, lambda_1) = Arc::get_mut(self)
                .expect("expected state to have no other refs")
                .fill_route(delta_2, &path, spill_price.unwrap_or_default())
                .await
                .unwrap();
            assert_eq!(lambda_1.asset_id, trading_pair.asset_1());
            assert_eq!(lambda_2.asset_id, trading_pair.asset_2());
            let lambda_1 = lambda_1.amount;
            let lambda_2 = lambda_2.amount;
            // TODO: don't we need to loop here to spill over and use up as much unfilled remaining assets as possible?
            tracing::debug!(?lambda_1, ?lambda_2, ?unfilled_1, ?unfilled_2);
            (lambda_1, lambda_2, true)
        } else {
            (0u64.into(), 0u64.into(), false)
        };

        let (lambda_1_1, lambda_2_2, lambda_2_1, lambda_1_2) = if success {
            (0u64.into(), 0u64.into(), lambda_2, lambda_1)
        } else {
            (delta_1, delta_2, 0u64.into(), 0u64.into())
        };
        Ok(())
    }
}

impl<T: PositionManager> RouteAndFill for T {}
