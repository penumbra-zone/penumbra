use penumbra_asset::{asset, Balance, Value};
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
}

#[cfg(test)]
mod tests {
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

        // The circuit breaker should not trip.
        assert!(value_circuit_breaker.check().is_ok());

        // If the same amount of gn is taken out of the position, the circuit breaker should not trip.
        value_circuit_breaker.tally(-new_b);
        assert!(value_circuit_breaker.check().is_ok());

        // But if there's ever a negative amount of gn in the position, the circuit breaker should trip.
        let one_b = Balance::from(Value {
            asset_id: pair.asset_2,
            amount: Amount::from(1u64),
        });
        value_circuit_breaker.tally(-one_b);
        assert!(value_circuit_breaker.check().is_err());
    }
}
