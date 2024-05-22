use cnidarium::StateWrite;

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

pub(crate) trait Chandelier: StateWrite {
    fn record_position_execution(
        &mut self,
        prev_state: &Position,
        new_state: &Position,
        trading_pair: &DirectedTradingPair,
    );
    fn record_swap_execution(&mut self, swap: &SwapExecution);
    fn finalize(&mut self) -> anyhow::Result<CandlestickData>;
}
