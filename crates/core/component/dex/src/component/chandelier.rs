use anyhow::Ok;
use anyhow::{Context as _, Result};

use cnidarium::{StateRead, StateWrite};
use futures::{StreamExt, TryStreamExt as _};
use penumbra_num::fixpoint::U128x128;
use penumbra_proto::{DomainType, StateReadProto, StateWriteProto};
use penumbra_sct::component::clock::EpochRead as _;
use tonic::async_trait;

use crate::event::EventCandlestickData;
use crate::{lp::position::Position, state_key::candlesticks, DirectedTradingPair, SwapExecution};

use crate::CandlestickData;

#[async_trait]
pub trait CandlestickRead: StateRead {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_candlestick(
        &self,
        trading_pair: &DirectedTradingPair,
        height: u64,
    ) -> Result<Option<CandlestickData>> {
        self.nonverifiable_get(
            candlesticks::data::by_pair_and_height(trading_pair, height).as_bytes(),
        )
        .await
    }

    async fn candlesticks(
        &self,
        trading_pair: &DirectedTradingPair,
        start_height: u64,
        limit: usize,
    ) -> Result<Vec<CandlestickData>> {
        let prefix = candlesticks::data::by_pair(&trading_pair);
        let start_height_key = format!("{:020}", start_height).as_bytes().to_vec();
        tracing::trace!(
            ?prefix,
            ?start_height,
            "searching for candlesticks from starting height"
        );

        let range = self
            .nonverifiable_range_raw(Some(prefix.as_bytes()), start_height_key..)
            .context("error forming range query")?;

        range
            .take(limit)
            .and_then(|(_k, v)| async move {
                CandlestickData::decode(v.as_ref()).context("error deserializing candlestick")
            })
            .try_collect()
            .await
    }
}
impl<T: StateRead + ?Sized> CandlestickRead for T {}

#[async_trait]
pub trait Chandelier: StateWrite {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn record_position_execution(
        &mut self,
        prev_state: &Position,
        new_state: &Position,
    ) -> Result<()> {
        // Redundant check that we do not record no-op executions.
        if prev_state == new_state {
            return Ok(());
        }

        // Determine the directionality of the trade.
        // If the reserves of asset 1 increased and asset 2 decreased, the trade was for asset 1 -> asset 2.
        // If the reserves of asset 2 increased and asset 1 decreased, the trade was for asset 2 -> asset 1.
        let trading_pair = prev_state.phi.pair;
        let directed_trading_pair = if new_state.reserves_for(trading_pair.asset_1)
            > prev_state.reserves_for(trading_pair.asset_1)
        {
            DirectedTradingPair::new(trading_pair.asset_1, trading_pair.asset_2)
        } else {
            DirectedTradingPair::new(trading_pair.asset_2, trading_pair.asset_1)
        };

        let mut block_executions = self
            .block_executions_by_pair(&directed_trading_pair)
            .clone();

        // The execution occurred at the price of the previous state.
        let execution_price = prev_state
            .phi
            .orient_start(directed_trading_pair.end)
            .context("recording position execution failed, position missing an end = asset 2")?
            .effective_price();

        // The volume can be found by the change in reserves of the input asset.
        let direct_volume = (new_state
            .reserves_for(directed_trading_pair.start)
            .context("missing reserves")?
            - prev_state
                .reserves_for(directed_trading_pair.start)
                .context("missing reserves")?)
        .into();

        tracing::debug!(
            ?directed_trading_pair,
            ?execution_price,
            ?direct_volume,
            "record position execution"
        );
        block_executions.push_back((execution_price, Some(direct_volume), None));
        self.put_block_executions_by_pair(&directed_trading_pair, block_executions);

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn record_swap_execution(&mut self, swap: &SwapExecution) {
        // Don't record a swap execution if the output amount was zero.
        // This is not an superfluous check, as the swap execution could really
        // have zero output e.g. in the case of a dust swap.
        if swap.output.amount == 0u32.into() || swap.input.amount == 0u32.into() {
            tracing::debug!(?swap, "skipping swap execution");
            return;
        }

        let trading_pair: DirectedTradingPair = DirectedTradingPair {
            start: swap.input.asset_id,
            end: swap.output.asset_id,
        };
        let mut block_executions = self.block_executions_by_pair(&trading_pair).clone();

        let execution_price = U128x128::ratio(swap.output.amount, swap.input.amount)
            .expect("input amount is not zero");

        // The volume is the amount of the input asset.
        let swap_volume = swap.input.amount.into();

        tracing::debug!(
            ?trading_pair,
            ?execution_price,
            ?swap_volume,
            "record swap execution"
        );
        block_executions.push_back((execution_price, None, Some(swap_volume)));
        self.put_block_executions_by_pair(&trading_pair, block_executions);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn finalize_block_candlesticks(&mut self) -> Result<()> {
        let height = self.get_block_height().await?;

        // Fetch all the executions for the block.
        let block_executions = self.block_executions();

        for (trading_pair, block_executions) in block_executions.iter() {
            // Since the block executions are stored in order as they occurred during the block,
            // we can iterate through them to create the candlestick data.
            let mut open = None;
            let mut close = 0.0;
            let mut low = f64::INFINITY;
            let mut high = 0.0;
            let mut swap_volume = 0.0;
            let mut direct_volume = 0.0;

            // Create summary data on a per-trading pair basis.
            for execution in block_executions {
                let (price, direct, swap) = execution;

                let price: f64 = (*price).into();

                if open.is_none() {
                    open = Some(price);
                }

                close = price;

                if price > high {
                    high = price;
                }

                if price < low {
                    low = price;
                }

                if let Some(direct) = direct {
                    direct_volume += f64::from(*direct);
                }

                if let Some(swap) = swap {
                    swap_volume += f64::from(*swap);
                }
            }

            // Store summary data in non-verifiable storage.
            let candlestick = CandlestickData {
                height,
                open: open.unwrap_or(0.0),
                close,
                high,
                low,
                direct_volume,
                swap_volume,
            };
            tracing::debug!(
                ?height,
                ?trading_pair,
                ?candlestick,
                "finalizing candlestick"
            );
            self.nonverifiable_put(
                candlesticks::data::by_pair_and_height(&trading_pair, height).into(),
                candlestick,
            );
            self.record_proto(
                EventCandlestickData {
                    pair: *trading_pair,
                    stick: candlestick,
                }
                .to_proto(),
            )
        }

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> Chandelier for T {}

#[async_trait]
trait Inner: StateWrite {
    #[tracing::instrument(level = "debug", skip(self))]
    fn block_executions(
        &self,
    ) -> im::HashMap<DirectedTradingPair, im::Vector<(U128x128, Option<U128x128>, Option<U128x128>)>>
    {
        self.object_get(candlesticks::object::block_executions())
            .unwrap_or_default()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn block_executions_by_pair(
        &self,
        trading_pair: &DirectedTradingPair,
    ) -> im::Vector<(U128x128, Option<U128x128>, Option<U128x128>)> {
        let new = im::Vector::new();
        let block_executions_map = self.block_executions();
        block_executions_map
            .get(trading_pair)
            .unwrap_or_else(|| &new)
            .clone()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn put_block_executions_by_pair(
        &mut self,
        trading_pair: &DirectedTradingPair,
        block_executions: im::Vector<(U128x128, Option<U128x128>, Option<U128x128>)>,
    ) {
        let mut block_executions_map = self.block_executions();
        block_executions_map.insert(trading_pair.clone(), block_executions);
        self.object_put(
            candlesticks::object::block_executions(),
            block_executions_map,
        );
    }
}
impl<T: StateWrite + ?Sized> Inner for T {}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use cnidarium::{ArcStateDeltaExt as _, StateDelta, TempStorage};
    use cnidarium_component::Component as _;
    use penumbra_asset::asset;
    use penumbra_sct::{component::clock::EpochManager as _, epoch::Epoch};
    use tendermint::abci;

    use crate::{
        component::{
            router::create_buy, tests::TempStorageExt as _, Dex, PositionManager as _,
            SwapDataRead, SwapDataWrite,
        },
        DirectedUnitPair,
    };

    use super::*;

    #[tokio::test]
    /// Perform basic tests of the chandelier.
    async fn chandelier_basic() -> anyhow::Result<()> {
        let _ = tracing_subscriber::fmt::try_init();
        let storage = TempStorage::new().await?.apply_minimal_genesis().await?;

        let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
        let mut state_tx = state.try_begin_transaction().unwrap();

        state_tx.put_block_height(0);
        state_tx.put_epoch_by_height(
            0,
            penumbra_sct::epoch::Epoch {
                index: 0,
                start_height: 0,
            },
        );
        state_tx.apply();

        storage.commit(Arc::try_unwrap(state).unwrap()).await?;
        let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

        // Create a single position and execute a swap against it.
        // We would expect to see direct flow and swap flow equal to each
        // other, and the price from the position for open/close/high/low.

        let penumbra = asset::Cache::with_known_assets()
            .get_unit("penumbra")
            .unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

        let pair_gn_penumbra = DirectedUnitPair::new(gn.clone(), penumbra.clone());

        // Create a single 1:2 gn:penumbra position (i.e. buy 1 gn at 2 penumbra).
        let mut state_tx = state.try_begin_transaction().unwrap();
        let buy_1 = create_buy(pair_gn_penumbra.clone(), 1u64.into(), 2u64.into());
        state_tx.open_position(buy_1).await.unwrap();
        state_tx.apply();

        // Now we should be able to fill a 1:1 gn:penumbra swap.
        let trading_pair = pair_gn_penumbra.into_directed_trading_pair().into();

        let mut swap_flow = state.swap_flow(&trading_pair);

        assert!(trading_pair.asset_1() == penumbra.id());

        // Add the amount of each asset being swapped to the batch swap flow.
        swap_flow.0 += 0u32.into();
        swap_flow.1 += gn.value(1u32.into()).amount;

        // Accumulate it into the batch swap flow for the trading pair.
        // Since this is currently empty this is the same as setting it.
        Arc::get_mut(&mut state)
            .unwrap()
            .accumulate_swap_flow(&trading_pair, swap_flow.clone())
            .await
            .unwrap();

        let height = 0u64;

        // End the block so the chandelier is generated
        let end_block = abci::request::EndBlock {
            height: height.try_into().unwrap(),
        };
        Dex::end_block(&mut state, &end_block).await;

        storage.commit(Arc::try_unwrap(state).unwrap()).await?;

        // Begin a new block and have a few more positions and swaps
        let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
        // set the epoch for the next block
        let mut state_tx = state.try_begin_transaction().unwrap();
        let height = 1u64;
        state_tx.put_block_height(height);
        state_tx.put_epoch_by_height(
            height,
            Epoch {
                index: 0,
                start_height: 0,
            },
        );
        state_tx.apply();

        // Check if the candlestick is set for height 0
        assert!(
            state
                .get_candlestick(&pair_gn_penumbra.into_directed_trading_pair(), 0u64)
                .await
                .unwrap()
                .is_some(),
            "candlestick exists for height 0"
        );

        let cs = state
            .get_candlestick(&pair_gn_penumbra.into_directed_trading_pair(), 0u64)
            .await
            .unwrap()
            .unwrap();

        let one_gn = gn.value(1u32.into());
        let base_gn = gn.base();
        let direct_volume: U128x128 = cs.direct_volume.try_into().unwrap();
        let swap_volume: U128x128 = cs.swap_volume.try_into().unwrap();
        assert_eq!(cs.height, 0u64, "height is 0");
        assert_eq!(cs.open, 2.0, "open price is 2.0");
        assert_eq!(cs.close, 2.0, "close price is 2.0");
        assert_eq!(cs.high, 2.0, "high price is 2.0");
        assert_eq!(cs.low, 2.0, "low price is 2.0");
        assert_eq!(
            base_gn.value(direct_volume.try_into().unwrap()),
            one_gn,
            "direct volume is 1 gn"
        );
        assert_eq!(
            base_gn.value(swap_volume.try_into().unwrap()),
            one_gn,
            "swap volume is 1 gn"
        );

        // Create a single 1:2 gn:penumbra position (i.e. buy 1 gn at 2 penumbra).
        let mut state_tx = state.try_begin_transaction().unwrap();
        let buy_1 = create_buy(pair_gn_penumbra.clone(), 1u64.into(), 2u64.into());
        state_tx.open_position(buy_1).await.unwrap();
        state_tx.apply();

        // Open a higher-priced position.
        // Create a single 1:1 gn:penumbra position (i.e. buy 1 gn at 1 penumbra).
        let mut state_tx = state.try_begin_transaction().unwrap();
        let buy_2 = create_buy(pair_gn_penumbra.clone(), 1u64.into(), 1u64.into());
        state_tx.open_position(buy_2).await.unwrap();
        state_tx.apply();

        // A swap for gn -> penumbra should first apply against the 1:2 position
        // resulting in an opening price of 2.0
        let mut swap_flow = state.swap_flow(&trading_pair);

        assert!(trading_pair.asset_1() == penumbra.id());

        // Add the amount of each asset being swapped to the batch swap flow.
        swap_flow.0 += 0u32.into();
        // Swap 2 gn into penumbra, meaning each position is filled.
        swap_flow.1 += gn.value(2u32.into()).amount;

        // Accumulate it into the batch swap flow for the trading pair.
        // Since this is currently empty this is the same as setting it.
        Arc::get_mut(&mut state)
            .unwrap()
            .accumulate_swap_flow(&trading_pair, swap_flow.clone())
            .await
            .unwrap();

        // End the block so the chandelier is generated
        let end_block = abci::request::EndBlock {
            height: height.try_into().unwrap(),
        };
        Dex::end_block(&mut state, &end_block).await;
        storage.commit(Arc::try_unwrap(state).unwrap()).await?;

        // Begin a new block and have a few more positions and swaps
        let state = Arc::new(StateDelta::new(storage.latest_snapshot()));
        // Check if the candlestick is set for height 0
        assert!(
            state
                .get_candlestick(&pair_gn_penumbra.into_directed_trading_pair(), height)
                .await
                .unwrap()
                .is_some(),
            "candlestick exists for height 1"
        );

        let cs = state
            .get_candlestick(&pair_gn_penumbra.into_directed_trading_pair(), height)
            .await
            .unwrap()
            .unwrap();

        let two_gn = gn.value(2u32.into());
        let base_gn = gn.base();
        let direct_volume: U128x128 = cs.direct_volume.try_into().unwrap();
        let swap_volume: U128x128 = cs.swap_volume.try_into().unwrap();
        assert_eq!(cs.height, 1u64, "height is 1");
        assert_eq!(cs.open, 2.0, "open price is 2.0");
        assert_eq!(cs.close, 1.5, "close price is 1.5");
        assert_eq!(cs.high, 2.0, "high price is 2.0");
        assert_eq!(cs.low, 1.0, "low price is 1.0");
        assert_eq!(
            base_gn.value(direct_volume.try_into().unwrap()),
            two_gn,
            "direct volume is 2 gn"
        );
        assert_eq!(
            base_gn.value(swap_volume.try_into().unwrap()),
            two_gn,
            "swap volume is 2 gn"
        );
        Ok(())
    }
}
