use std::future;
use std::{pin::Pin, sync::Arc};

use anyhow::Result;
use async_stream::try_stream;
use async_trait::async_trait;
use cnidarium::{EscapedByteSlice, StateRead, StateWrite};
use futures::Stream;
use futures::StreamExt;
use penumbra_asset::{asset, Balance, Value};
use penumbra_num::Amount;
use penumbra_proto::DomainType;
use penumbra_proto::{StateReadProto, StateWriteProto};

use crate::circuit_breaker::ValueCircuitBreaker;
use crate::lp::position::State;
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
        // We pull the position from the state unconditionally, since we will
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

        // Update the value circuit breaker's aggregate account.
        self.update_position_aggregate_value(&position, &prev)
            .await?;

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
        // Clone the fixed candidates Arc so it can be moved into the stream filter's future.
        let fc = fixed_candidates.clone();
        let mut dynamic_candidates = self
            .ordered_routable_assets(&from)
            .filter(move |c| {
                future::ready(!fc.contains(c.as_ref().expect("failed to fetch candidate")))
            })
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
        self.nonverifiable_prefix_raw(&prefix)
            .map(|entry| match entry {
                Ok((_, v)) => Ok(asset::Id::decode(&*v)?),
                Err(e) => Err(e),
            })
            .boxed()
    }
}

impl<T: StateWrite + ?Sized> PositionManager for T {}

#[async_trait]
pub(crate) trait Inner: StateWrite {
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

    /// Updates the nonverifiable liquidity indices given a [`Position`] in the direction specified by the [`DirectedTradingPair`].
    /// An [`Option<Position>`] may be specified to allow for the case where a position is being updated.
    async fn update_liquidity_index(
        &mut self,
        pair: DirectedTradingPair,
        position: &Position,
        prev: &Option<Position>,
    ) -> Result<()> {
        tracing::debug!(?pair, "updating available liquidity indices");

        let (new_a_from_b, current_a_from_b) = match (position.state, prev) {
            (State::Opened, None) => {
                // Add the new position's contribution to the index, no cancellation of the previous version necessary.

                // Query the current available liquidity for this trading pair, or zero if the trading pair
                // has no current liquidity.
                let current_a_from_b = self
                    .nonverifiable_get_raw(&state_key::internal::routable_assets::a_from_b(&pair))
                    .await?
                    .map(|bytes| {
                        Amount::from_be_bytes(
                            bytes
                                .try_into()
                                .expect("liquidity index amount can always be parsed"),
                        )
                    })
                    .unwrap_or_default();

                // Use the new reserves to compute `new_position_contribution`,
                // the amount of asset A contributed by the position (i.e. the reserves of asset A).
                let new_position_contribution = position
                    .reserves_for(pair.start)
                    .expect("specified position should match provided trading pair");

                // Compute `new_A_from_B`.
                let new_a_from_b =
                    // Add the contribution from the updated version.
                    current_a_from_b.saturating_add(&new_position_contribution);

                tracing::debug!(?pair, current_liquidity = ?current_a_from_b, ?new_position_contribution, "newly opened position, adding contribution to existing available liquidity for trading pair");

                (new_a_from_b, current_a_from_b)
            }
            (State::Opened, Some(prev)) => {
                // Add the new position's contribution to the index, deleting the previous version's contribution.

                // Query the current available liquidity for this trading pair, or zero if the trading pair
                // has no current liquidity.
                let current_a_from_b = self
                    .nonverifiable_get_raw(&state_key::internal::routable_assets::a_from_b(&pair))
                    .await?
                    .map(|bytes| {
                        Amount::from_be_bytes(
                            bytes
                                .try_into()
                                .expect("liquidity index amount can always be parsed"),
                        )
                    })
                    .unwrap_or_default();

                // Use the previous reserves to compute `prev_position_contribution` (denominated in asset_1).
                let prev_position_contribution = prev
                    .reserves_for(pair.start)
                    .expect("specified position should match provided trading pair");

                // Use the new reserves to compute `new_position_contribution`,
                // the amount of asset A contributed by the position (i.e. the reserves of asset A).
                let new_position_contribution = position
                    .reserves_for(pair.start)
                    .expect("specified position should match provided trading pair");

                // Compute `new_A_from_B`.
                let new_a_from_b =
                // Subtract the previous version of the position's contribution to represent that position no longer
                // being correct, and add the contribution from the updated version.
                (current_a_from_b.saturating_sub(&prev_position_contribution)).saturating_add(&new_position_contribution);

                tracing::debug!(?pair, current_liquidity = ?current_a_from_b, ?new_position_contribution, ?prev_position_contribution, "updated position, adding new contribution and subtracting previous contribution to existing available liquidity for trading pair");

                (new_a_from_b, current_a_from_b)
            }
            (State::Closed, Some(prev)) => {
                // Compute the previous contribution and erase it from the current index

                // Query the current available liquidity for this trading pair, or zero if the trading pair
                // has no current liquidity.
                let current_a_from_b = self
                    .nonverifiable_get_raw(&state_key::internal::routable_assets::a_from_b(&pair))
                    .await?
                    .map(|bytes| {
                        Amount::from_be_bytes(
                            bytes
                                .try_into()
                                .expect("liquidity index amount can always be parsed"),
                        )
                    })
                    .unwrap_or_default();

                // Use the previous reserves to compute `prev_position_contribution` (denominated in asset_1).
                let prev_position_contribution = prev
                    .reserves_for(pair.start)
                    .expect("specified position should match provided trading pair");

                // Compute `new_A_from_B`.
                let new_a_from_b =
                // Subtract the previous version of the position's contribution to represent that position no longer
                // being correct, and since the updated version is Closed, it has no contribution.
                current_a_from_b.saturating_sub(&prev_position_contribution);

                tracing::debug!(?pair, current_liquidity = ?current_a_from_b, ?prev_position_contribution, "closed position, subtracting previous contribution to existing available liquidity for trading pair");

                (new_a_from_b, current_a_from_b)
            }
            (State::Withdrawn { .. }, _) | (State::Closed, None) => {
                // The position already went through the `Closed` state or was opened in the `Closed` state, so its contribution has already been subtracted.
                return Ok(());
            }
        };

        // Delete the existing key for this position if the reserve amount has changed.
        if new_a_from_b != current_a_from_b {
            self.nonverifiable_delete(
                state_key::internal::routable_assets::key(&pair.start, current_a_from_b).to_vec(),
            );
        }

        // Write the new key indicating that asset B is routable from asset A with `new_a_from_b` liquidity.
        self.nonverifiable_put_raw(
            state_key::internal::routable_assets::key(&pair.start, new_a_from_b).to_vec(),
            pair.end.encode_to_vec(),
        );
        tracing::debug!(start = ?pair.start, end = ?pair.end, "marking routable from start -> end");

        // Write the new lookup index storing `new_a_from_b` for this trading pair.
        self.nonverifiable_put_raw(
            state_key::internal::routable_assets::a_from_b(&pair).to_vec(),
            new_a_from_b.to_be_bytes().to_vec(),
        );
        tracing::debug!(available_liquidity = ?new_a_from_b, ?pair, "marking available liquidity for trading pair");

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
        self.update_liquidity_index(DirectedTradingPair::new(a, b), position, prev_position)
            .await?;
        // B -> A
        self.update_liquidity_index(DirectedTradingPair::new(b, a), position, prev_position)
            .await?;

        Ok(())
    }

    /// Tracks the total token supply deposited in positions for all assets to ensure
    /// asset value conservation (i.e. that more assets can't come out of positions than
    /// were deposited).
    async fn update_position_aggregate_value(
        &mut self,
        position: &Position,
        prev_position: &Option<Position>,
    ) -> Result<()> {
        tracing::debug!(
            ?position,
            ?prev_position,
            "updating position aggregate value"
        );

        // Find the difference in the amounts of assets A and B, based on the state of the position being stored,
        // and the previous state of the position.
        let (net_change_for_a, net_change_for_b) = match (position.state, prev_position) {
            (State::Opened, None) => {
                // The position is newly opened, so the change is the full amount of assets A and B.

                // Use the new reserves to compute `new_position_contribution`,
                // the amount of asset A contributed by the position (i.e. the reserves of asset A).
                let pair = position.phi.pair;
                let new_a = position
                    .reserves_for(pair.asset_1)
                    .expect("specified position should match provided trading pair");
                let new_b = position
                    .reserves_for(pair.asset_2)
                    .expect("specified position should match provided trading pair");

                let new_a = Balance::from(Value {
                    asset_id: pair.asset_1,
                    amount: new_a,
                });
                let new_b = Balance::from(Value {
                    asset_id: pair.asset_2,
                    amount: new_b,
                });
                (new_a, new_b)
            }
            (State::Opened, Some(prev)) => {
                // The position is still open however the reserves have changed, so the change is the difference
                // between the previous reserves and the new reserves.
                let pair = position.phi.pair;
                let new_a = Balance::from(Value {
                    asset_id: pair.asset_1,
                    amount: position
                        .reserves_for(pair.asset_1)
                        .expect("specified position should match provided trading pair"),
                });
                let new_b = Balance::from(Value {
                    asset_id: pair.asset_2,
                    amount: position
                        .reserves_for(pair.asset_2)
                        .expect("specified position should match provided trading pair"),
                });
                let old_a = Balance::from(Value {
                    asset_id: pair.asset_1,
                    amount: prev
                        .reserves_for(pair.asset_1)
                        .expect("specified position should match provided trading pair"),
                });
                let old_b = Balance::from(Value {
                    asset_id: pair.asset_2,
                    amount: prev
                        .reserves_for(pair.asset_2)
                        .expect("specified position should match provided trading pair"),
                });

                (new_a - old_a, new_b - old_b)
            }
            (State::Closed, Some(prev)) => {
                // The previous amount of assets A and B should be subtracted from the aggregate value.

                let pair = position.phi.pair;
                let old_a = prev
                    .reserves_for(pair.asset_1)
                    .expect("specified position should match provided trading pair");
                let old_b = prev
                    .reserves_for(pair.asset_2)
                    .expect("specified position should match provided trading pair");

                let old_a = Balance::from(Value {
                    asset_id: pair.asset_1,
                    amount: old_a,
                });
                let old_b = Balance::from(Value {
                    asset_id: pair.asset_2,
                    amount: old_b,
                });
                // The position is closed, so the change is the negative of the previous reserves.
                (-old_a, -old_b)
            }
            (State::Withdrawn { .. }, _) | (State::Closed, None) => {
                // The position already went through the `Closed` state or was opened in the `Closed` state, so its contribution has already been subtracted.
                return Ok(());
            }
        };

        tracing::debug!(
            ?position,
            ?net_change_for_a,
            ?net_change_for_b,
            "updating position assets' aggregate balances"
        );

        let mut value_circuit_breaker: ValueCircuitBreaker = match self
            .nonverifiable_get_raw(state_key::aggregate_value().as_bytes())
            .await
            .expect("able to retrieve value circuit breaker from nonverifiable storage")
        {
            Some(bytes) => serde_json::from_slice(&bytes).expect(
                "able to deserialize stored value circuit breaker from nonverifiable storage",
            ),
            None => ValueCircuitBreaker::default(),
        };

        // Add the change to the value circuit breaker for assets A and B.
        value_circuit_breaker.tally(net_change_for_a);
        value_circuit_breaker.tally(net_change_for_b);

        // Confirm that the value circuit breaker is still within the limits.
        // This call will panic if the value circuit breaker detects inflation.
        value_circuit_breaker.check()?;

        // Store the value circuit breaker back to nonconsensus storage with the updated tallies.
        self.nonverifiable_put_raw(
            state_key::aggregate_value().as_bytes().to_vec(),
            serde_json::to_vec(&value_circuit_breaker)
                .expect("able to serialize value circuit breaker for nonverifiable storage"),
        );

        Ok(())
    }
}
impl<T: StateWrite + ?Sized> Inner for T {}
