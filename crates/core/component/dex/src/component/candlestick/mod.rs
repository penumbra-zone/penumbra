use cnidarium::StateWrite;
use tonic::async_trait;

use crate::{lp::position::Position, DirectedTradingPair, SwapExecution};

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

#[async_trait]
pub(crate) trait Chandelier: StateWrite {
    async fn record_position_execution(
        &mut self,
        prev_state: &Position,
        new_state: &Position,
        trading_pair: &DirectedTradingPair,
    ) {
        todo!()
    }

    async fn record_swap_execution(&mut self, swap: &SwapExecution) {
        todo!()
    }

    async fn finalize_block_candlesticks(&mut self) -> anyhow::Result<CandlestickData> {
        todo!()
    }
}

impl<T: StateWrite + ?Sized> Chandelier for T {}
