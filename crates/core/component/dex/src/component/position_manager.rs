use std::{collections::BTreeSet, iter::FromIterator, pin::Pin, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use futures::StreamExt;
use penumbra_crypto::{asset, Amount};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{EscapedByteSlice, StateRead, StateWrite};

use crate::{
    lp::position::{self, Position},
    state_key, DirectedTradingPair,
};

#[async_trait]
pub trait PositionRead: StateRead {
    /// Return a stream of all [`position::Metadata`] available.
    fn all_positions(
        &self,
    ) -> Pin<Box<dyn Stream<Item = Result<position::Position>> + Send + 'static>> {
        let prefix = state_key::all_positions();
        self.prefix(prefix)
            .map(|entry| match entry {
                Ok((_, metadata)) => {
                    tracing::debug!(?metadata, "found position");
                    Ok(metadata)
                }
                Err(e) => Err(e),
            })
            .boxed()
    }

    /// Returns a stream of [`position::Id`] ordered by effective price.
    fn positions_by_price(
        &self,
        pair: &DirectedTradingPair,
    ) -> Pin<Box<dyn Stream<Item = Result<position::Id>> + Send + 'static>> {
        let prefix = state_key::internal::price_index::prefix(pair);
        tracing::trace!(prefix = ?EscapedByteSlice(&prefix), "searching for positions by price");
        self.nonconsensus_prefix_raw(&prefix)
            .map(|entry| match entry {
                Ok((k, _)) => {
                    let raw_id = <&[u8; 32]>::try_from(&k[103..135])?.to_owned();
                    Ok(position::Id(raw_id))
                }
                Err(e) => Err(e),
            })
            .boxed()
    }

    async fn position_by_id(&self, id: &position::Id) -> Result<Option<position::Position>> {
        self.get(&state_key::position_by_id(id)).await
    }

    async fn check_position_id_unused(&self, id: &position::Id) -> Result<()> {
        match self.get_raw(&state_key::position_by_id(id)).await? {
            Some(_) => Err(anyhow::anyhow!("position id {:?} already used", id)),
            None => Ok(()),
        }
    }

    async fn best_position(
        &self,
        pair: &DirectedTradingPair,
    ) -> Result<Option<position::Position>> {
        let mut positions_by_price = self.positions_by_price(pair);
        match positions_by_price.next().await.transpose()? {
            Some(id) => self.position_by_id(&id).await,
            None => Ok(None),
        }
    }

    /// Fetch the list of pending position closures.
    fn pending_position_closures(&self) -> im::Vector<position::Id> {
        self.object_get(state_key::pending_position_closures())
            .unwrap_or_default()
    }
}
impl<T: StateRead + ?Sized> PositionRead for T {}

/// Manages liquidity positions within the chain state.
#[async_trait]
pub trait PositionManager: StateWrite + PositionRead {
    /// Queues a position to be closed at the end of the block, after batch execution.
    fn queue_close_position(&mut self, id: position::Id) {
        let mut to_close = self.pending_position_closures();
        to_close.push_back(id);
        self.object_put(state_key::pending_position_closures(), to_close);
    }

    async fn close_queued_positions(&mut self) -> Result<()> {
        let to_close = self.pending_position_closures();
        for id in to_close {
            let mut position = self
                .position_by_id(&id)
                .await?
                .expect("value balance mechanism should have ensured position exists");

            position.state = position::State::Closed;
            self.put_position(position).await?;
        }
        self.object_delete(state_key::pending_position_closures());
        Ok(())
    }

    /// Writes a position to the state, updating all necessary indexes.
    #[tracing::instrument(level = "debug", skip(self, position), fields(id = ?position.id()))]
    async fn put_position(&mut self, position: position::Position) -> Result<()> {
        let id = position.id();
        tracing::debug!(?position, "processing position");
        // Clear any existing indexes of the position, since changes to the
        // reserves or the position state might have invalidated them.
        self.deindex_position_by_price(&position);

        // We pull the position from the state inconditionally, since we will
        // always need to update the position's liquidity index.
        let prev = self.position_by_id(&id).await?;

        let position = self.handle_limit_order(prev, position);

        // Only index the position's liquidity if it is active.
        if position.state == position::State::Opened {
            self.index_position_by_price(&position);
        }
        self.put(state_key::position_by_id(&id), position);

        Ok(())
    }

    /// Handle a limit order, inspecting it previous state to determine if it
    /// has been filled, and if so, marking it as closed. If the position is
    /// not a limit order, or has not been filled, it is returned unchanged.
    fn handle_limit_order(
        &self,
        prev_position: Option<position::Position>,
        position: Position,
    ) -> Position {
        match prev_position {
            Some(_) if position.close_on_fill => {
                // It's technically possible for a limit order to be partially filled,
                // and unfilled on the other side. In this case, we would close it prematurely.
                // However, because of the arbitrage dynamics we expect that in practice an order
                // gets completely filled or not at all.
                if position.reserves.r1 == Amount::zero() || position.reserves.r2 == Amount::zero()
                {
                    Position {
                        state: position::State::Closed,
                        ..position
                    }
                } else {
                    position
                }
            }
            _ => position,
        }
    }

    /// Returns the list of candidate assets to route through for a trade from `from`.
    /// Combines a list of fixed candidates with a list of liquidity-based candidates.
    /// This ensures that the fixed candidates are always considered, minimizing
    /// the risk of attacks on routing.
    async fn candidate_set(
        &self,
        _from: asset::Id,
        fixed_candidates: Arc<Vec<asset::Id>>,
    ) -> Result<Vec<asset::Id>> {
        let candidates = BTreeSet::from_iter(fixed_candidates.iter().cloned());
        // TODO: do dynamic candidate selection based on liquidity (tracked by #2750)
        // Idea: each block, compute the per-asset candidate set and store it
        // in the object store as a BTreeMap.
        Ok(candidates.into_iter().collect())
    }
}

impl<T: StateWrite + ?Sized> PositionManager for T {}

#[async_trait]
pub(super) trait Inner: StateWrite {
    fn index_position_by_price(&mut self, position: &position::Position) {
        let (pair, phi) = (position.phi.pair, &position.phi);
        let id = position.id();
        if position.reserves.r2 != 0u64.into() {
            // Index this position for trades FROM asset 1 TO asset 2, since the position has asset 2 to give out.
            let pair12 = DirectedTradingPair {
                start: pair.asset_1(),
                end: pair.asset_2(),
            };
            let phi12 = phi.component.clone();
            self.nonconsensus_put_raw(
                state_key::internal::price_index::key(&pair12, &phi12, &id),
                vec![],
            );
            tracing::debug!("indexing position for 1=>2 trades");
        }

        if position.reserves.r1 != 0u64.into() {
            // Index this position for trades FROM asset 2 TO asset 1, since the position has asset 1 to give out.
            let pair21 = DirectedTradingPair {
                start: pair.asset_2(),
                end: pair.asset_1(),
            };
            let phi21 = phi.component.flip();
            self.nonconsensus_put_raw(
                state_key::internal::price_index::key(&pair21, &phi21, &id),
                vec![],
            );
            tracing::debug!("indexing position for 2=>1 trades");
        }
    }

    fn deindex_position_by_price(&mut self, position: &Position) {
        let id = position.id();
        tracing::debug!("deindexing position");
        let pair12 = DirectedTradingPair {
            start: position.phi.pair.asset_1(),
            end: position.phi.pair.asset_2(),
        };
        let phi12 = position.phi.component.clone();
        let pair21 = DirectedTradingPair {
            start: position.phi.pair.asset_2(),
            end: position.phi.pair.asset_1(),
        };
        let phi21 = position.phi.component.flip();
        self.nonconsensus_delete(state_key::internal::price_index::key(&pair12, &phi12, &id));
        self.nonconsensus_delete(state_key::internal::price_index::key(&pair21, &phi21, &id));
    }
}
impl<T: StateWrite + ?Sized> Inner for T {}
