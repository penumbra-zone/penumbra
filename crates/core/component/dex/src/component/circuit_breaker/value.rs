use anyhow::{anyhow, Result};
use cnidarium::StateWrite;
use penumbra_asset::Value;
use penumbra_num::Amount;
use penumbra_proto::{StateReadProto, StateWriteProto};
use tonic::async_trait;

use crate::{event, state_key};

/// Tracks the aggregate value of deposits in the DEX.
#[async_trait]
pub trait ValueCircuitBreaker: StateWrite {
    /// Credits a deposit into the DEX.
    async fn vcb_credit(&mut self, value: Value) -> Result<()> {
        let balance: Amount = self
            .get(&state_key::value_balance(&value.asset_id))
            .await?
            .unwrap_or_default();
        let new_balance = balance
            .checked_add(&value.amount)
            .ok_or_else(|| anyhow!("overflowed balance while crediting value circuit breaker"))?;
        self.put(state_key::value_balance(&value.asset_id), new_balance);

        self.record_proto(event::vcb_credit(value.asset_id, balance, new_balance));
        Ok(())
    }

    /// Debits a deposit from the DEX.
    async fn vcb_debit(&mut self, value: Value) -> Result<()> {
        let balance: Amount = self
            .get(&state_key::value_balance(&value.asset_id))
            .await?
            .unwrap_or_default();
        let new_balance = balance
            .checked_sub(&value.amount)
            .ok_or_else(|| anyhow!("underflowed balance while debiting value circuit breaker"))?;
        self.put(state_key::value_balance(&value.asset_id), new_balance);

        self.record_proto(event::vcb_debit(value.asset_id, balance, new_balance));
        Ok(())
    }
}

impl<T: StateWrite + ?Sized> ValueCircuitBreaker for T {}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::component::position_manager::Inner as _;
    use crate::component::router::HandleBatchSwaps as _;
    use crate::component::{StateReadExt as _, StateWriteExt as _};
    use crate::lp::plan::PositionWithdrawPlan;
    use crate::{
        component::{router::create_buy, tests::TempStorageExt},
        state_key, DirectedUnitPair,
    };
    use crate::{BatchSwapOutputData, PositionOpen};
    use cnidarium::{ArcStateDeltaExt as _, StateDelta, TempStorage};
    use cnidarium_component::ActionHandler as _;
    use penumbra_asset::asset;
    use penumbra_num::Amount;
    use penumbra_proto::StateWriteProto as _;
    use penumbra_sct::component::clock::EpochManager as _;
    use penumbra_sct::component::source::SourceContext as _;
    use penumbra_sct::epoch::Epoch;

    use super::*;

    #[tokio::test]
    async fn value_circuit_breaker() -> anyhow::Result<()> {
        let _ = tracing_subscriber::fmt::try_init();
        let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
        let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
        let mut state_tx = state.try_begin_transaction().unwrap();

        let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
        let test_usd = asset::Cache::with_known_assets()
            .get_unit("test_usd")
            .unwrap();

        // A credit followed by a debit of the same amount should succeed.
        // Credit 100 gm.
        state_tx.vcb_credit(gm.value(100u64.into())).await?;
        // Credit 100 gn.
        state_tx.vcb_credit(gn.value(100u64.into())).await?;

        // Debit 100 gm.
        state_tx.vcb_debit(gm.value(100u64.into())).await?;
        // Debit 100 gn.
        state_tx.vcb_debit(gn.value(100u64.into())).await?;

        // Debiting an additional gm should fail.
        assert!(state_tx.vcb_debit(gm.value(1u64.into())).await.is_err());

        // Debiting an asset that hasn't been credited should also fail.
        assert!(state_tx
            .vcb_debit(test_usd.value(1u64.into()))
            .await
            .is_err());

        Ok(())
    }

    #[tokio::test]
    async fn position_value_circuit_breaker() -> anyhow::Result<()> {
        let _ = tracing_subscriber::fmt::try_init();
        let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
        let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
        let mut state_tx = state.try_begin_transaction().unwrap();

        let height = 1;

        // 1. Simulate BeginBlock

        state_tx.put_epoch_by_height(
            height,
            Epoch {
                index: 0,
                start_height: 0,
            },
        );
        state_tx.put_block_height(height);
        state_tx.apply();

        let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

        let pair_1 = DirectedUnitPair::new(gm.clone(), gn.clone());

        let one = 1u64.into();
        let price1 = one;
        // Create a position buying 1 gm with 1 gn (i.e. reserves will be 1gn).
        let buy_1 = create_buy(pair_1.clone(), 1u64.into(), price1);

        // Create the PositionOpen action
        let pos_open = PositionOpen {
            position: buy_1.clone(),
        };

        // Execute the PositionOpen action.
        pos_open.check_stateless(()).await?;
        pos_open.check_historical(state.clone()).await?;
        let mut state_tx = state.try_begin_transaction().unwrap();
        state_tx.put_mock_source(1u8);
        pos_open.check_and_execute(&mut state_tx).await?;
        state_tx.apply();

        // Set the output data for the block to 1 gn and 0 gm.
        // This should not error, the circuit breaker should not trip.
        let mut state_tx = state.try_begin_transaction().unwrap();
        state_tx
            .set_output_data(
                BatchSwapOutputData {
                    delta_1: 0u64.into(),
                    delta_2: 1u64.into(),
                    lambda_1: 0u64.into(),
                    lambda_2: 0u64.into(),
                    unfilled_1: 0u64.into(),
                    unfilled_2: 0u64.into(),
                    height: 1,
                    trading_pair: pair_1.into_directed_trading_pair().into(),
                    epoch_starting_height: 0,
                },
                None,
                None,
            )
            .await?;

        // Pretend the position was overfilled.

        // Wipe out the gm value in the circuit breaker, so that any outflows should trip it.
        state_tx.put(state_key::value_balance(&gm.id()), Amount::from(0u64));

        // Create the PositionWithdraw action
        let pos_withdraw_plan = PositionWithdrawPlan {
            position_id: buy_1.id(),
            reserves: buy_1.reserves,
            sequence: 1,
            pair: pair_1.into_directed_trading_pair().into(),
            rewards: vec![],
        };

        let pos_withdraw = pos_withdraw_plan.position_withdraw();

        // Execute the PositionWithdraw action.
        pos_withdraw.check_stateless(()).await?;
        pos_withdraw.check_historical(state.clone()).await?;
        let mut state_tx = state.try_begin_transaction().unwrap();
        state_tx.put_mock_source(1u8);
        // This should error, since there is no balance available to withdraw the position.
        assert!(pos_withdraw.check_and_execute(&mut state_tx).await.is_err());
        state_tx.apply();

        Ok(())
    }

    #[tokio::test]
    #[should_panic(expected = "underflowed balance while debiting value circuit breaker")]
    async fn batch_swap_circuit_breaker() {
        let _ = tracing_subscriber::fmt::try_init();
        let storage = TempStorage::new()
            .await
            .expect("able to create storage")
            .apply_minimal_genesis()
            .await
            .expect("able to apply genesis");
        let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
        let mut state_tx = state.try_begin_transaction().unwrap();

        let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

        let pair_1 = DirectedUnitPair::new(gm.clone(), gn.clone());

        // Manually put a position without calling `put_position` so that the
        // circuit breaker is not aware of the position's value. Then, handling a batch
        // swap that fills against this position should result in an error.
        let one = 1u64.into();
        let price1 = one;
        // Create a position buying 1 gm with 1 gn (i.e. reserves will be 1gn).
        let buy_1 = create_buy(pair_1.clone(), 1u64.into(), price1);

        let id = buy_1.id();

        let position = buy_1;
        state_tx.index_position_by_price(&position, &position.id());
        state_tx
            .update_available_liquidity(&position, &None)
            .await
            .expect("able to update liquidity");
        state_tx.put(state_key::position_by_id(&id), position);

        // Now there's a position in the state, but the circuit breaker is not aware of it.
        let trading_pair = pair_1.into_directed_trading_pair().into();
        let mut swap_flow = state_tx.swap_flow(&trading_pair);

        assert!(trading_pair.asset_1() == gm.id());

        // Add the amount of each asset being swapped to the batch swap flow.
        swap_flow.0 += gm.value(5u32.into()).amount;
        swap_flow.1 += 0u32.into();

        // Set the batch swap flow for the trading pair.
        state_tx
            .put_swap_flow(&trading_pair, swap_flow.clone())
            .await
            .unwrap();
        state_tx.apply();

        let routing_params = state.routing_params().await.unwrap();
        // This call should panic due to the outflow of gn not being covered by the circuit breaker.
        state
            .handle_batch_swaps(trading_pair, swap_flow, 0, 0, routing_params)
            .await
            .expect("unable to process batch swaps");
    }
}
