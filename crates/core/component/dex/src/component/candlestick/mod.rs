use cnidarium::StateWrite;
use tonic::async_trait;

use crate::{lp::position::Position, state_key, DirectedTradingPair, SwapExecution};

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
    #[tracing::instrument(level = "debug", skip(self))]
    async fn record_position_execution(
        &mut self,
        prev_state: &Position,
        new_state: &Position,
        trading_pair: &DirectedTradingPair,
    ) {
        let mut executions = self.block_position_executions();
        // TODO: create some composite summary data here rather than just storing the positions.
        executions.push_back((prev_state.clone(), new_state.clone(), trading_pair.clone()));
        self.object_put(state_key::block_position_executions(), executions);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn record_swap_execution(&mut self, swap: &SwapExecution) {
        let mut swap_executions = self.block_swap_executions();
        swap_executions.push_back(swap.clone());
        self.object_put(state_key::block_swap_executions(), swap_executions);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn finalize_block_candlesticks(&mut self) -> anyhow::Result<CandlestickData> {
        // Fetch all the position executions for the block.
        // Fetch all the swap executions for the block.
        // Create summary data on a per-trading pair basis.
        todo!()
    }
}

impl<T: StateWrite + ?Sized> Chandelier for T {}

#[async_trait]
trait Inner: StateWrite {
    #[tracing::instrument(level = "debug", skip(self))]
    fn block_position_executions(&self) -> im::Vector<(Position, Position, DirectedTradingPair)> {
        self.object_get(state_key::block_position_executions())
            .unwrap_or_default()
    }

    fn block_swap_executions(&self) -> im::Vector<SwapExecution> {
        self.object_get(state_key::block_swap_executions())
            .unwrap_or_default()
    }
}
impl<T: StateWrite + ?Sized> Inner for T {}
