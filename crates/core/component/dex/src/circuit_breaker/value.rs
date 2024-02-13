use penumbra_asset::{asset::Id, Balance, Value};
use penumbra_num::Amount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValueCircuitBreaker {
    balance: Balance,
}

impl ValueCircuitBreaker {
    pub fn tally(&mut self, balance: Balance) {
        self.balance += balance;
    }

    pub fn check(&self) -> anyhow::Result<()> {
        // No assets should ever be "required" by the circuit breaker's
        // internal balance tracking, only "provided".
        if let Some(r) = self.balance.required().next() {
            return Err(anyhow::anyhow!(
                "balance for asset {} is negative: -{}",
                r.asset_id,
                r.amount
            ));
        }

        Ok(())
    }

    pub fn available(&self, asset_id: Id) -> Value {
        self.balance
            .provided()
            .find(|b| b.asset_id == asset_id)
            .unwrap_or(Value {
                asset_id,
                amount: Amount::from(0u64),
            })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::component::position_manager::Inner as _;
    use crate::component::router::{HandleBatchSwaps as _, RoutingParams};
    use crate::component::{StateReadExt as _, StateWriteExt as _};
    use crate::{
        component::{router::limit_buy, tests::TempStorageExt, PositionManager as _},
        state_key, DirectedUnitPair,
    };
    use cnidarium::{
        ArcStateDeltaExt as _, StateDelta, StateRead as _, StateWrite as _, TempStorage,
    };
    use penumbra_asset::{asset, Value};
    use penumbra_num::Amount;
    use penumbra_proto::StateWriteProto as _;
    use rand_core::OsRng;

    use crate::{
        lp::{position::Position, Reserves},
        DirectedTradingPair,
    };

    use super::*;

    // Ideally the update_position_aggregate_value in the PositionManager would be used
    // but this is simpler for a quick unit test.

    #[test]
    fn value_circuit_breaker() {
        let mut value_circuit_breaker = ValueCircuitBreaker::default();

        let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

        let pair = DirectedTradingPair::new(gm.id(), gn.id());
        let reserves_1 = Reserves {
            r1: 0u64.into(),
            r2: 120_000u64.into(),
        };

        // A position with 120_000 gn and 0 gm.
        let position_1 = Position::new(
            OsRng,
            pair,
            9u32,
            1_200_000u64.into(),
            1_000_000u64.into(),
            reserves_1,
        );

        // Track the position in the circuit breaker.
        let pair = position_1.phi.pair;
        let new_a = position_1
            .reserves_for(pair.asset_1)
            .expect("specified position should match provided trading pair");
        let new_b = position_1
            .reserves_for(pair.asset_2)
            .expect("specified position should match provided trading pair");

        let new_a = Balance::from(Value {
            asset_id: pair.asset_1,
            amount: new_a,
        });
        let new_b = Balance::from(Value {
            asset_id: pair.asset_2,
            amount: new_b,
        });
        value_circuit_breaker.tally(new_a);
        value_circuit_breaker.tally(new_b.clone());

        assert!(value_circuit_breaker.available(pair.asset_1).amount == 0u64.into());
        assert!(value_circuit_breaker.available(pair.asset_2).amount == 120_000u64.into());

        // The circuit breaker should not trip.
        assert!(value_circuit_breaker.check().is_ok());

        // If the same amount of gn is taken out of the position, the circuit breaker should not trip.
        value_circuit_breaker.tally(-new_b);
        assert!(value_circuit_breaker.check().is_ok());

        assert!(value_circuit_breaker.available(pair.asset_1).amount == 0u64.into());
        assert!(value_circuit_breaker.available(pair.asset_2).amount == 0u64.into());

        // But if there's ever a negative amount of gn in the position, the circuit breaker should trip.
        let one_b = Balance::from(Value {
            asset_id: pair.asset_2,
            amount: Amount::from(1u64),
        });
        value_circuit_breaker.tally(-one_b);
        assert!(value_circuit_breaker.check().is_err());
        assert!(value_circuit_breaker.available(pair.asset_1).amount == 0u64.into());
        assert!(value_circuit_breaker.available(pair.asset_2).amount == 0u64.into());
    }

    #[tokio::test]
    async fn position_value_circuit_breaker() -> anyhow::Result<()> {
        let _ = tracing_subscriber::fmt::try_init();
        let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
        let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
        let mut state_tx = state.try_begin_transaction().unwrap();

        let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

        let pair_1 = DirectedUnitPair::new(gm.clone(), gn.clone());

        let one = 1u64.into();
        let price1 = one;
        // Create a position buying 1 gm with 1 gn (i.e. reserves will be 1gn).
        let mut buy_1 = limit_buy(pair_1.clone(), 1u64.into(), price1);
        state_tx.put_position(buy_1.clone()).await.unwrap();

        // Update the position to buy 1 gm with 2 gn (i.e. reserves will be 2gn).
        buy_1.reserves.r2 = 2u64.into();
        state_tx.put_position(buy_1.clone()).await.unwrap();

        // Pretend the position has been filled against and flipped, so there's no
        // gn in the position and there is 2 gm.
        buy_1.reserves.r1 = 2u64.into();
        buy_1.reserves.r2 = 0u64.into();

        // This should not error, the circuit breaker should not trip.
        state_tx.put_position(buy_1.clone()).await.unwrap();

        // Pretend the position was overfilled.
        let mut value_circuit_breaker: ValueCircuitBreaker = match state_tx
            .nonverifiable_get_raw(state_key::aggregate_value().as_bytes())
            .await
            .expect("able to retrieve value circuit breaker from nonverifiable storage")
        {
            Some(bytes) => serde_json::from_slice(&bytes).expect(
                "able to deserialize stored value circuit breaker from nonverifiable storage",
            ),
            None => panic!("should have a circuit breaker present"),
        };

        // Wipe out the value in the circuit breaker, so that any outflows should trip it.
        value_circuit_breaker.balance = Balance::default();
        state_tx.nonverifiable_put_raw(
            state_key::aggregate_value().as_bytes().to_vec(),
            serde_json::to_vec(&value_circuit_breaker)
                .expect("able to serialize value circuit breaker for nonverifiable storage"),
        );

        // This should error, since there is no balance available to close out the position.
        buy_1.state = crate::lp::position::State::Closed;
        assert!(state_tx.put_position(buy_1).await.is_err());

        Ok(())
    }

    #[tokio::test]
    #[should_panic(expected = "balance for asset")]
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
        let buy_1 = limit_buy(pair_1.clone(), 1u64.into(), price1);

        let id = buy_1.id();

        let position = state_tx.handle_limit_order(&None, buy_1);
        state_tx.index_position_by_price(&position);
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
        state_tx.put_swap_flow(&trading_pair, swap_flow.clone());
        state_tx.apply();

        // This call should panic due to the outflow of gn not being covered by the circuit breaker.
        state
            .handle_batch_swaps(trading_pair, swap_flow, 0, 0, RoutingParams::default())
            .await
            .expect("unable to process batch swaps");
    }
}
