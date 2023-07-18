use std::{pin::Pin, sync::Arc};

use anyhow::Result;
use async_stream::try_stream;
use async_trait::async_trait;
use futures::Stream;
use futures::StreamExt;
use penumbra_asset::asset;
use penumbra_asset::Value;
use penumbra_num::Amount;
use penumbra_proto::DomainType;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{EscapedByteSlice, StateRead, StateWrite};

use crate::lp::Reserves;
use crate::{
    lp::position::{self, Position},
    state_key, DirectedTradingPair,
};

const DYNAMIC_ASSET_LIMIT: usize = 10;

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
        self.nonverifiable_prefix_raw(&prefix)
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
    /// Close a position by id, removing it from the state.
    /// # Errors
    /// Returns an error if the position does not exist.
    async fn close_position_by_id(&mut self, id: &position::Id) -> Result<()> {
        tracing::debug!(?id, "closing position, first fetch it");
        let mut position = self
            .position_by_id(id)
            .await
            .expect("fetching position should not fail")
            .ok_or_else(|| anyhow::anyhow!("position not found"))?;

        tracing::debug!(?id, "position found, close it");
        position.state = position::State::Closed;
        self.put_position(position).await?;
        Ok(())
    }

    /// Queues a position to be closed at the end of the block, after batch execution.
    fn queue_close_position(&mut self, id: position::Id) {
        let mut to_close = self.pending_position_closures();
        to_close.push_back(id);
        self.object_put(state_key::pending_position_closures(), to_close);
    }

    /// Close all positions that have been queued for closure.
    async fn close_queued_positions(&mut self) -> () {
        let to_close = self.pending_position_closures();
        for id in to_close {
            match self.close_position_by_id(&id).await {
                Ok(()) => tracing::debug!(?id, "position closed"),
                // The position was already closed, which in and of itself is not an error.
                // It's possible that the position was closed by the engine, for example
                // because it was a limit-order.
                Err(e) => tracing::debug!(?id, "failed to close position: {}", e),
            }
        }
        self.object_delete(state_key::pending_position_closures());
    }

    /// Writes a position to the state, updating all necessary indexes.
    #[tracing::instrument(level = "debug", skip(self, position), fields(id = ?position.id()))]
    async fn put_position(&mut self, position: position::Position) -> Result<()> {
        let id = position.id();
        tracing::debug!(?position, "fetch position's previous state from storage");
        // We pull the position from the state inconditionally, since we will
        // always need to update the position's liquidity index.
        let prev = self
            .position_by_id(&id)
            .await
            .expect("fetching position should not fail");

        // Clear any existing indexes of the position, since changes to the
        // reserves or the position state might have invalidated them.
        self.deindex_position_by_price(&position);

        let position = self.handle_limit_order(&prev, position);

        // Only index the position's liquidity if it is active.
        if position.state == position::State::Opened {
            self.index_position_by_price(&position);
        }

        // Update the available liquidity for this position's trading pair.
        self.update_available_liquidity(&position, &prev).await?;

        self.put(state_key::position_by_id(&id), position);
        Ok(())
    }

    /// Handle a limit order, inspecting it previous state to determine if it
    /// has been filled, and if so, marking it as closed. If the position is
    /// not a limit order, or has not been filled, it is returned unchanged.
    fn handle_limit_order(
        &self,
        prev_position: &Option<position::Position>,
        position: Position,
    ) -> Position {
        let id = position.id();
        match prev_position {
            Some(_) if position.close_on_fill => {
                // It's technically possible for a limit order to be partially filled,
                // and unfilled on the other side. In this case, we would close it prematurely.
                // However, because of the arbitrage dynamics we expect that in practice an order
                // gets completely filled or not at all.
                if position.reserves.r1 == Amount::zero() || position.reserves.r2 == Amount::zero()
                {
                    tracing::debug!(?id, "limit order filled, setting state to closed");
                    Position {
                        state: position::State::Closed,
                        ..position
                    }
                } else {
                    tracing::debug!(?id, "limit order partially filled, keeping open");
                    position
                }
            }
            None if position.close_on_fill => {
                tracing::debug!(?id, "detected a newly opened limit order");
                position
            }
            _ => position,
        }
    }

    /// Returns the list of candidate assets to route through for a trade from `from`.
    /// Combines a list of fixed candidates with a list of liquidity-based candidates.
    /// This ensures that the fixed candidates are always considered, minimizing
    /// the risk of attacks on routing.
    fn candidate_set(
        &self,
        from: asset::Id,
        fixed_candidates: Arc<Vec<asset::Id>>,
    ) -> Pin<Box<dyn Stream<Item = Result<asset::Id>> + Send>> {
        let mut dynamic_candidates = self
            .ordered_routable_assets(&from)
            .take(DYNAMIC_ASSET_LIMIT);
        try_stream! {
            // First stream the fixed candidates, so those can be processed while the dynamic candidates are fetched.
            for candidate in fixed_candidates.iter() {
                yield candidate.clone();
            }

            // Yield the liquidity-based candidates. Note that this _may_ include some assets already included in the fixed set.
            while let Some(candidate) = dynamic_candidates
                .next().await {
                    yield candidate.expect("failed to fetch candidate");
            }
        }
        .boxed()
    }

    /// Returns a stream of [`asset::Id`] routable from a given asset, ordered by liquidity.
    fn ordered_routable_assets(
        &self,
        from: &asset::Id,
    ) -> Pin<Box<dyn Stream<Item = Result<asset::Id>> + Send + 'static>> {
        let prefix = state_key::internal::routable_assets::prefix(from);
        tracing::trace!(prefix = ?EscapedByteSlice(&prefix), "searching for routable assets by liquidity");
        self.nonconsensus_prefix_raw(&prefix)
            .map(|entry| match entry {
                Ok((_, v)) => Ok(asset::Id::decode(&*v)?),
                Err(e) => Err(e),
            })
            .boxed()
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
            self.nonverifiable_put_raw(
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
            self.nonverifiable_put_raw(
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
        self.nonverifiable_delete(state_key::internal::price_index::key(&pair12, &phi12, &id));
        self.nonverifiable_delete(state_key::internal::price_index::key(&pair21, &phi21, &id));
    }

    async fn _update_available_liquidity(
        &mut self,
        trading_pair: DirectedTradingPair,
        position: &Position,
        prev_position: &Option<Position>,
    ) -> Result<()> {
        // Query the current available liquidity for this trading pair, or zero if the trading pair
        // has no current liquidity.
        let current_a_from_b = self
            .nonconsensus_get_raw(&state_key::internal::routable_assets::a_from_b(
                &trading_pair,
            ))
            .await?
            .map(|bytes| {
                Amount::from_be_bytes(
                    bytes
                        .try_into()
                        .expect("invalid a_from_b stored in nonconsensus"),
                )
            })
            .unwrap_or_default();

        // Get the current reserves of this position, or zero if the position is newly added.
        let current_reserves = prev_position
            .as_ref()
            .map(|prev| prev.reserves.clone())
            .unwrap_or(Reserves {
                r1: Amount::zero(),
                r2: Amount::zero(),
            });

        // Use the current reserves to compute `current_position_contribution` (denominated in asset_1).
        let current_position_contribution = match prev_position {
            Some(prev_position) => {
                // Return the amount of asset A purchasable with all reserves of asset B.
                prev_position
                    .phi
                    .fill(
                        Value {
                            asset_id: trading_pair.end,
                            amount: prev_position
                                .reserves_for(trading_pair.end)
                                .unwrap_or_default(),
                        },
                        &current_reserves,
                    )?
                    .2
                    .amount
            }
            None => Amount::zero(),
        };

        // Use the new reserves to compute `new_position_contribution`,
        // the amount of asset A purchasable with all reserves of asset B.
        let new_position_contribution = match position.state {
            position::State::Opened => {
                // Return the amount of asset A purchasable with all reserves of asset B.
                position
                    .phi
                    .fill(
                        Value {
                            asset_id: trading_pair.end,
                            amount: position.reserves_for(trading_pair.end).unwrap_or_default(),
                        },
                        &position.reserves,
                    )?
                    .2
                    .amount
            }
            _ => Amount::zero(),
        };

        // Compute `new_A_from_B`.
        let new_a_from_b =
            // Subtract the current version of the position's contribution to represent that position no longer
            // being correct, and add the contribution from the updated version.
            (current_a_from_b - current_position_contribution) + new_position_contribution;

        // Delete the existing key for this position if the reserve amount has changed.
        if new_a_from_b != current_a_from_b {
            self.nonconsensus_delete(
                state_key::internal::routable_assets::key(&trading_pair.start, current_a_from_b)
                    .to_vec(),
            );
        }

        // Write the new key indicating that asset B is routable from asset A with `new_a_from_b` liquidity.
        self.nonconsensus_put_raw(
            state_key::internal::routable_assets::key(&trading_pair.start, new_a_from_b).to_vec(),
            trading_pair.end.encode_to_vec(),
        );

        // Write the new lookup index storing `new_a_from_b` for this trading pair.
        self.nonconsensus_put_raw(
            state_key::internal::routable_assets::a_from_b(&trading_pair).to_vec(),
            new_a_from_b.to_be_bytes().to_vec(),
        );
        Ok(())
    }

    async fn update_available_liquidity(
        &mut self,
        position: &Position,
        prev_position: &Option<Position>,
    ) -> Result<()> {
        // Since swaps may be performed in either direction, the available liquidity indices
        // need to be calculated and stored for both the A -> B and B -> A directions.
        let (a, b) = (position.phi.pair.asset_1(), position.phi.pair.asset_2());

        // A -> B
        self._update_available_liquidity(DirectedTradingPair::new(a, b), position, prev_position)
            .await?;
        // B -> A
        self._update_available_liquidity(DirectedTradingPair::new(b, a), position, prev_position)
            .await?;

        Ok(())
    }
}
impl<T: StateWrite + ?Sized> Inner for T {}
