use anyhow::bail;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};

use crate::lp::position::{self, Position};
use crate::state_key::engine;
use crate::TradingPair;
use anyhow::Result;

#[async_trait]
pub(crate) trait PositionCounterRead: StateRead {
    /// Returns the number of position for a [`TradingPair`].
    /// If there were no counter initialized for a given pair, this default to zero.
    async fn get_position_count(&self, trading_pair: &TradingPair) -> u32 {
        let path = engine::counter::num_positions::by_trading_pair(trading_pair);
        self.get_position_count_from_key(path).await
    }

    async fn get_position_count_from_key(&self, path: [u8; 99]) -> u32 {
        let Some(raw_count) = self
            .nonverifiable_get_raw(&path)
            .await
            .expect("no deserialization failure")
        else {
            return 0;
        };

        // This is safe because we only increment the counter via [`Self::increase_position_counter`].
        let raw_count: [u8; 4] = raw_count
            .try_into()
            .expect("position counter is at most two bytes");
        u32::from_be_bytes(raw_count)
    }
}

impl<T: StateRead + ?Sized> PositionCounterRead for T {}

#[async_trait]
pub(crate) trait PositionCounter: StateWrite {
    async fn update_trading_pair_position_counter(
        &mut self,
        prev_state: &Option<Position>,
        new_state: &Position,
    ) -> Result<()> {
        use position::State::*;
        let trading_pair = new_state.phi.pair;
        match (prev_state.as_ref().map(|p| p.state), new_state.state) {
            // Increment the counter whenever a new position is opened
            (None, Opened) => {
                let _ = self.increment_position_counter(&trading_pair).await?;
            }
            // Decrement the counter whenever an opened position is closed
            (Some(Opened), Closed) => {
                let _ = self.decrement_position_counter(&trading_pair).await?;
            }
            // Other state transitions don't affect the opened position counter
            _ => {}
        }
        Ok(())
    }
}
impl<T: StateWrite + ?Sized> PositionCounter for T {}

trait Inner: StateWrite {
    /// Increment the number of position for a [`TradingPair`].
    /// Returns the updated total, or an error if overflow occurred.
    async fn increment_position_counter(&mut self, trading_pair: &TradingPair) -> Result<u32> {
        let path = engine::counter::num_positions::by_trading_pair(trading_pair);
        let prev = self.get_position_count_from_key(path).await;

        let Some(new_total) = prev.checked_add(1) else {
            bail!("incrementing position counter would overflow")
        };
        self.nonverifiable_put_raw(path.to_vec(), new_total.to_be_bytes().to_vec());
        Ok(new_total)
    }

    /// Decrement the number of positions for a [`TradingPair`], unless it would underflow.
    /// Returns the updated total, or an error if underflow occurred.
    async fn decrement_position_counter(&mut self, trading_pair: &TradingPair) -> Result<u32> {
        let path = engine::counter::num_positions::by_trading_pair(trading_pair);
        let prev = self.get_position_count_from_key(path).await;

        let Some(new_total) = prev.checked_sub(1) else {
            bail!("decrementing position counter would underflow")
        };
        self.nonverifiable_put_raw(path.to_vec(), new_total.to_be_bytes().to_vec());
        Ok(new_total)
    }
}

impl<T: StateWrite + ?Sized> Inner for T {}

// For some reason, `rust-analyzer` is complaining about used imports.
// Silence the warnings until I find a fix.
#[allow(unused_imports)]
mod tests {
    use cnidarium::{StateDelta, StateWrite, TempStorage};
    use penumbra_sdk_asset::{asset::REGISTRY, Value};

    use crate::component::position_manager::counter::{
        Inner, PositionCounter, PositionCounterRead,
    };
    use crate::state_key::engine;
    use crate::TradingPair;

    #[tokio::test]
    /// Test that we can detect overflows and that they are handled properly: increment is ignored / no crash.
    async fn test_no_overflow() -> anyhow::Result<()> {
        let asset_a = REGISTRY.parse_denom("upenumbra").unwrap().id();
        let asset_b = REGISTRY.parse_denom("pizza").unwrap().id();
        let trading_pair = TradingPair::new(asset_a, asset_b);

        let storage = TempStorage::new().await?;
        let mut delta = StateDelta::new(storage.latest_snapshot());
        let path = engine::counter::num_positions::by_trading_pair(&trading_pair);
        // Manually set the counter to the maximum value
        delta.nonverifiable_put_raw(path.to_vec(), u32::MAX.to_be_bytes().to_vec());

        // Check that the counter is at the maximum value
        let total = delta.get_position_count(&trading_pair).await;
        assert_eq!(total, u32::MAX);

        // Check that we can handle an overflow
        assert!(delta
            .increment_position_counter(&trading_pair)
            .await
            .is_err());
        assert_eq!(delta.get_position_count(&trading_pair).await, u32::MAX);

        Ok(())
    }

    #[tokio::test]
    /// Test that we can detect underflow and that they are handled properly: decrement is ignored / no crash.
    async fn test_no_underflow() -> anyhow::Result<()> {
        let asset_a = REGISTRY.parse_denom("upenumbra").unwrap().id();
        let asset_b = REGISTRY.parse_denom("pizza").unwrap().id();
        let trading_pair = TradingPair::new(asset_a, asset_b);

        let storage = TempStorage::new().await?;
        let mut delta = StateDelta::new(storage.latest_snapshot());

        let maybe_total = delta.decrement_position_counter(&trading_pair).await;
        assert!(maybe_total.is_err());

        let counter = delta.get_position_count(&trading_pair).await;
        assert_eq!(counter, 0u32);
        Ok(())
    }
}
