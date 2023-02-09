use anyhow::{Error, Result};
use std::collections::{BTreeMap, HashSet};
use tokio::task::JoinSet;

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

/// The maximum number of hops allowed in a trade.
/// Prevents exploding the number of paths that need to be considered.
const MAX_HOPS: usize = 5;

/// Represent the distance between two assets in a weighted graph.
type TradeDistance = f64;

/// Finds the best route for a trade, based on a Bellman-Ford algorithm
/// across dex trading pairs with available liquidity.
pub struct TradeRouter<T: PositionRead> {
    // TODO: maybe these should be in non-consensus storage
    /// Maintains a map of best distances between assets.
    pub optimal_paths: BTreeMap<asset::Id, TradeDistance>,
    /// Maintains a map of the best predecessor (best priced position) for each asset.
    pub predecessors: BTreeMap<asset::Id, Option<asset::Id>>,
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
            predecessors: BTreeMap::new(),
            state,
            positions,
        })
    }

    /// Finds the best route for a trade, based on a Bellman-Ford algorithm
    /// across dex trading pairs with available liquidity.
    ///
    /// This takes place against a state fork directly, rather than constructing separate in-memory structures.
    pub async fn find_route(
        &mut self,
        trading_pair: &DirectedTradingPair,
        amount: &Amount,
    ) -> Result<Path> {
        // The distance from the source asset to itself is always 0.
        self.optimal_paths.entry(trading_pair.start).or_insert(0.0);

        // Initialize predecessors for the source and target assets.
        self.predecessors.entry(trading_pair.start).or_default();
        self.predecessors.entry(trading_pair.end).or_default();

        // For storing each unique asset.
        let mut known_assets = HashSet::new();
        known_assets.insert(trading_pair.start.0);
        known_assets.insert(trading_pair.end.0);

        // The distance from the source asset to all other assets is initially infinite.
        // TODO: use a JoinSet to parallelize this
        for position in self.state.positions().await?.iter() {
            // Skip positions that are not opened.
            if position.state != position::State::Opened {
                continue;
            }

            let position_pair = position.position.phi.pair;

            // If there's not a distance from the source asset to either asset of the position's trading pair,
            // initialize it to infinite.
            if position_pair.asset_1() != trading_pair.start {
                self.optimal_paths
                    .entry(position_pair.asset_1())
                    .or_insert(f64::INFINITY);
            }

            if position_pair.asset_2() != trading_pair.start {
                self.optimal_paths
                    .entry(position_pair.asset_2())
                    .or_insert(f64::INFINITY);
            }

            // Initialize all predecessors to None.
            self.predecessors
                .entry(position_pair.asset_1())
                .or_default();
            self.predecessors
                .entry(position_pair.asset_2())
                .or_default();

            // Insert the position's trading pair's assets into the known assets set.
            known_assets.insert(position_pair.asset_1().0);
            known_assets.insert(position_pair.asset_2().0);
        }

        // Perform edge relaxation |V| - 1 times (where |V| is the number of unique assets present within positions).
        for _ in 0..known_assets.len() - 1 {
            // TODO: account for MAX_HOPS, binning, dust position exclusion
            // For each position...
            for position in self.state.positions().await?.iter() {
                // Skip positions that are not opened.
                if position.state != position::State::Opened {
                    continue;
                }

                // If the distance to the destination can be shortened by taking the edge, update the optimal path.
                if *self
                    .optimal_paths
                    .get(&position.position.phi.pair.asset_1())
                    // Should be safe because all assets were initialized earlier
                    .expect("all assets should be initialized")
                    != f64::INFINITY
                    && self
                        .optimal_paths
                        .get(&position.position.phi.pair.asset_1())
                        .unwrap()
                        // TODO: this shouldn't be a simple addition, i think it needs to compose the two trading functions
                        + 
                            position.position.phi.component.effective_price()
                        < *self
                            .optimal_paths
                            .get(&position.position.phi.pair.asset_2())
                            .expect("all assets should be initialized")
                {
                    self.optimal_paths.insert(
                        position.position.phi.pair.asset_2(),
                    self
                        .optimal_paths
                        .get(&position.position.phi.pair.asset_1())
                        .unwrap()
                        // TODO: this shouldn't be a simple addition, i think it needs to compose the two trading functions
                        + 
                            position.position.phi.component.effective_price()
                    );
                    self.predecessors.insert(
                        position.position.phi.pair.asset_2(),
                        Some(position.position.phi.pair.asset_1()),
                    );
                }
            }
        }

        // Detect negative cycles.
        for position in self.state.positions().await?.iter() {
            // Skip positions that are not opened.
            if position.state != position::State::Opened {
                continue;
            }

            // If the destination gets a better price by taking the position, update the optimal path.
            if *self
                .optimal_paths
                .get(&position.position.phi.pair.asset_1())
                // Should be safe because all assets were initialized earlier
                .expect("all assets should be initialized")
                != f64::INFINITY
                && self
                    .optimal_paths
                    .get(&position.position.phi.pair.asset_1())
                    .unwrap()
                // TODO: should not be a simple addition, needs to compose the prices
                + 
                    position.position.phi.component.effective_price()
                < *self
                    .optimal_paths
                    .get(&position.position.phi.pair.asset_2())
                    .expect("all assets should be initialized")
            {
                return Err(anyhow::anyhow!("graph contains negative weight cycle"));
            }
        }
    
        // Calculate optimal path from start -> end
        // The path begins as 0-length, from start to itself, with no fee.
        let mut path = Path::new(trading_pair.start, trading_pair.start, TradingFunction::new(TradingPair::new(trading_pair.start, trading_pair.start), 0, amount.clone(), amount.clone())).expect("able to instantiate new path");
        let mut current = Some(trading_pair.start);

        loop {
            let pred = self.predecessors.get(&current.unwrap()).expect("predecessors initialized");
            if pred.is_none() {
                break;
            }

            // TODO: use correct amounts and fees
            path.extend(TradingFunction::new(TradingPair::new(pred.unwrap(), current.unwrap()), 0, 1.into(), 1.into()));
            current = pred.clone();
        }

        Ok(path)
    }
}


mod tests {
    use std::sync::Arc;

    use penumbra_storage::TempStorage;

    use crate::TempStorageExt;

    #[tokio::test]
    async fn test_simple() -> anyhow::Result<()> {
        // Create a storage backend for testing.
        let storage = TempStorage::new().await?.apply_default_genesis().await?;

        let mut state = Arc::new(storage.latest_state());

        // Test trading a source asset for itself.

        todo!();

        // Test a single position between a source asset and target asset.

        todo!();

        Ok(())
    }
}
