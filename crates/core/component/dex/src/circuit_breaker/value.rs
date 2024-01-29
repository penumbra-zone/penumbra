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

    pub fn assert_balance_invariant(&self) {
        // No assets should ever be "required" by the circuit breaker's
        // internal balance tracking, only "provided".
        if let Some(r) = self.balance.required().next() {
            assert!(
                false,
                "balance for asset {} is negative: -{}",
                r.asset_id, r.amount
            );
        }
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

    use crate::{
        component::{router::limit_buy, tests::TempStorageExt, PositionManager as _},
        DirectedUnitPair,
    };
    use cnidarium::{ArcStateDeltaExt as _, StateDelta, TempStorage};
    use penumbra_asset::{asset, Value};
    use penumbra_num::Amount;
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
        let result = std::panic::catch_unwind(|| value_circuit_breaker.assert_balance_invariant());
        assert!(result.is_ok());

        // If the same amount of gn is taken out of the position, the circuit breaker should not trip.
        value_circuit_breaker.tally(-new_b);
        let result = std::panic::catch_unwind(|| value_circuit_breaker.assert_balance_invariant());
        assert!(result.is_ok());

        assert!(value_circuit_breaker.available(pair.asset_1).amount == 0u64.into());
        assert!(value_circuit_breaker.available(pair.asset_2).amount == 0u64.into());

        // But if there's ever a negative amount of gn in the position, the circuit breaker should trip.
        let one_b = Balance::from(Value {
            asset_id: pair.asset_2,
            amount: Amount::from(1u64),
        });
        value_circuit_breaker.tally(-one_b);
        let result = std::panic::catch_unwind(|| value_circuit_breaker.assert_balance_invariant());
        assert!(result.is_err());
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

        // This should not panic, the circuit breaker should not trip.
        state_tx.put_position(buy_1.clone()).await.unwrap();

        // Pretend the position was overfilled.
        buy_1.reserves.r1 = 0u64.into();
        buy_1.reserves.r2 = 0u64.into();
        // This should panic
        state_tx.put_position(buy_1.clone()).await.unwrap();

        Ok(())
    }
}
