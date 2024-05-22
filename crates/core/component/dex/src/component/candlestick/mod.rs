use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};

use cnidarium::StateWrite;
use penumbra_num::fixpoint::U128x128;
use penumbra_proto::{core::component::dex::v1 as pb, DomainType};
use penumbra_sct::component::clock::EpochRead as _;
use tonic::async_trait;

use crate::{lp::position::Position, state_key, DirectedTradingPair, SwapExecution};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "pb::CandlestickData", into = "pb::CandlestickData")]
pub(crate) struct CandlestickData {
    /// The height of the candlestick data.
    height: u64,
    /// The first observed price during the block execution.
    open: f64,
    /// The last observed price during the block execution.
    close: f64,
    /// The highest observed price during the block execution.
    high: f64,
    /// The lowest observed price during the block execution.
    low: f64,
    /// The volume that traded "directly", during individual position executions.
    direct_volume: f64,
    /// The volume that traded as part of swaps, which could have traversed multiple routes.
    swap_volume: f64,
}

impl DomainType for CandlestickData {
    type Proto = pb::CandlestickData;
}

impl From<CandlestickData> for pb::CandlestickData {
    fn from(cd: CandlestickData) -> Self {
        Self {
            height: cd.height,
            open: cd.open,
            close: cd.close,
            high: cd.high,
            low: cd.low,
            direct_volume: cd.direct_volume,
            swap_volume: cd.swap_volume,
        }
    }
}

impl TryFrom<pb::CandlestickData> for CandlestickData {
    type Error = anyhow::Error;
    fn try_from(cd: pb::CandlestickData) -> Result<Self, Self::Error> {
        Ok(Self {
            height: cd.height,
            open: cd.open,
            close: cd.close,
            high: cd.high,
            low: cd.low,
            direct_volume: cd.direct_volume,
            swap_volume: cd.swap_volume,
        })
    }
}

#[async_trait]
pub trait Chandelier: StateWrite {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn record_position_execution(
        &mut self,
        prev_state: &Position,
        new_state: &Position,
        trading_pair: &DirectedTradingPair,
    ) -> Result<()> {
        let mut block_executions = self.block_executions_by_pair(trading_pair).clone();

        // The execution occurred at the price of the previous state.
        let execution_price = prev_state
            .phi
            .orient_start(trading_pair.start)
            .context("position has one end = asset 1")?
            .effective_price();

        // The volume can be found by the change in reserves of the input asset.
        let direct_volume = (prev_state
            .reserves_for(trading_pair.start)
            .context("missing reserves")?
            - new_state
                .reserves_for(trading_pair.start)
                .context("missing reserves")?)
        .into();

        block_executions.push_back((execution_price, Some(direct_volume), None));
        self.put_block_executions_by_pair(trading_pair, block_executions);

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn record_swap_execution(&mut self, swap: &SwapExecution) -> Result<()> {
        let trading_pair = DirectedTradingPair {
            start: swap.input.asset_id,
            end: swap.output.asset_id,
        };
        let mut block_executions = self.block_executions_by_pair(&trading_pair).clone();

        // The execution price is the output priced in terms of the input.
        let execution_price = U128x128::ratio(swap.output.amount, swap.input.amount)
            .context("denom unit is not 0")?;

        // The volume is the amount of the input asset.
        let swap_volume = swap.input.amount.into();

        block_executions.push_back((execution_price, None, Some(swap_volume)));
        self.put_block_executions_by_pair(&trading_pair, block_executions);

        Ok(())
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
            self.nonverifiable_put_raw(
                state_key::candlesticks::by_pair_and_height(&trading_pair, height).into(),
                candlestick.encode_to_vec(),
            );
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
        self.object_get(state_key::candlesticks::block_executions())
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
            state_key::candlesticks::block_executions(),
            block_executions_map,
        );
    }
}
impl<T: StateWrite + ?Sized> Inner for T {}
