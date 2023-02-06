use anyhow::{Error, Result};
use std::collections::BTreeMap;

use penumbra_crypto::{
    asset,
    dex::{
        execution::Path,
        lp::{position, TradingFunction},
        DirectedTradingPair, TradingPair,
    },
    Amount,
};

use super::position_manager::PositionRead;

/// Finds the best route for a trade, based on a Bellman-Ford algorithm
/// across dex trading pairs with available liquidity.
pub struct TradeRouter<T: PositionRead> {
    /// Maintains a map of optimal paths between assets.
    /// The outer `BTreeMap` is keyed by the starting asset, and the
    /// inner `BTreeMap` represents the optimal paths to each other asset.
    pub optimal_paths: BTreeMap<asset::Id, BTreeMap<asset::Id, Path>>,
    /// The `TradeRouter` needs to be able to read trading positions from state.
    state: T,
    /// Tracks known liquidity positions when the `TradeRouter` is constructed.
    positions: Vec<position::Metadata>,
}

impl<T: PositionRead> TradeRouter<T> {
    pub async fn new(state: T) -> Result<Self> {
        let positions = state.positions().await?;

        Ok(Self {
            optimal_paths: BTreeMap::new(),
            state,
            positions,
        })
    }

    /// Finds the best route for a trade, based on a Bellman-Ford algorithm
    /// across dex trading pairs with available liquidity.
    pub async fn find_route(
        &mut self,
        trading_pair: &DirectedTradingPair,
        amount: &Amount,
    ) -> Result<Path> {
        // https://www.programiz.com/dsa/bellman-ford-algorithm
        // First, construct a weighted graph based on the source asset.
        // The weighted graph will contain every asset that can be traded with the source asset,
        // including from multiple hops.
        let weighted_graph = self.construct_weighted_graph(trading_pair).await?;

        // The distance from the source asset to itself is always 0.
        self.optimal_paths
            .entry(trading_pair.start)
            .or_default()
            .insert(
                trading_pair.start,
                Path::new(
                    trading_pair.start,
                    trading_pair.start,
                    TradingFunction::new(
                        TradingPair::new(trading_pair.start, trading_pair.start),
                        // Can always trade 1:1 with self without fee
                        0,
                        *amount,
                        *amount,
                    ),
                )?,
            );

        // For all edges...
        // If the distance to the destination can be shortened by taking the edge, update the optimal path.
        unimplemented!()
    }

    async fn construct_weighted_graph(&self, trading_pair: &DirectedTradingPair) -> Result<()> {
        // Find all liquidity positions involving the source asset.
        let positions = self
            .state
            .positions()
            .await?
            .iter()
            .filter(|position| {
                position.state == position::State::Opened
                    && (position.position.phi.pair.asset_1() == trading_pair.start
                        || position.position.phi.pair.asset_2() == trading_pair.start)
            })
            .collect::<Vec<_>>();

        // For each trading pair, if the source asset is the first asset in the pair,
        // add the second asset to the graph.
        // If the source asset is the second asset in the pair, add the first asset to the graph.
        // If the source asset is not in the pair, skip the pair.
        // For each asset added to the graph, recursively construct the weighted graph for that asset.
        unimplemented!()
    }
}
