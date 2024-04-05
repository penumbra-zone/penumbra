use std::future;
use std::{pin::Pin, sync::Arc};

use anyhow::Result;
use async_stream::try_stream;
use async_trait::async_trait;
use cnidarium::{EscapedByteSlice, StateRead, StateWrite};
use futures::Stream;
use futures::StreamExt;
use penumbra_asset::{asset, Balance};
use penumbra_num::Amount;
use penumbra_proto::DomainType;
use penumbra_proto::{StateReadProto, StateWriteProto};

use crate::event;
use crate::lp::position::State;
use crate::lp::Reserves;
use crate::{
    component::position_counter::PositionCounter,
    component::ValueCircuitBreaker,
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
impl<T: StateRead + ?Sized> PositionRead for T {}

/// Manages liquidity positions within the chain state.
#[async_trait]
pub trait PositionManager: StateWrite + PositionRead {
    /// Close a position by id, removing it from the state.
    ///
    /// If the position is already closed, this is a no-op.
    ///
    /// # Errors
    ///
    /// Returns an error if the position does not exist.
    async fn close_position_by_id(&mut self, id: &position::Id) -> Result<()> {
        tracing::debug!(?id, "closing position, first fetch it");
        let prev_state = self
            .position_by_id(id)
            .await
            .expect("fetching position should not fail")
            .ok_or_else(|| anyhow::anyhow!("could not find position {} to close", id))?;

        anyhow::ensure!(
            matches!(
                prev_state.state,
                position::State::Opened | position::State::Closed,
            ),
            "attempted to close a position with state {:?}, expected Opened or Closed",
            prev_state.state
        );

        // Skip state updates if the position is already closed: to keep the position counter
        // accurate and skip unnecessary I/O.
        if prev_state.state == position::State::Closed {
            // A position could be closed multiple times e.g. it is queued for closure by the user
            // and preemptively closed by the DEX engine during filling.
            tracing::debug!(
                ?id,
                "position is already closed so we can skip state updates"
            );
            return Ok(());
        }

        let new_state = {
            let mut new_state = prev_state.clone();
            new_state.state = position::State::Closed;
            new_state
        };

        self.update_position(Some(prev_state), new_state).await?;

        Ok(())
    }

    /// Queues a position to be closed at the end of the block, after batch execution.
    fn queue_close_position(&mut self, id: position::Id) {
        let mut to_close = self.pending_position_closures();
        to_close.push_back(id);
        self.object_put(state_key::pending_position_closures(), to_close);
    }

    /// Close all positions that have been queued for closure.
    async fn close_queued_positions(&mut self) -> Result<()> {
        let to_close = self.pending_position_closures();
        for id in to_close {
            self.close_position_by_id(&id).await?;
        }
        self.object_delete(state_key::pending_position_closures());
        Ok(())
    }

    /// Opens a new position, updating all necessary indexes and checking for
    /// its nonexistence prior to being opened.
    ///
    /// # Errors
    /// This method returns an error if the position is malformed
    /// e.g. it is set to a state other than `Opened`
    ///  or, it specifies a position identifier already used by another position.
    ///
    /// An error can also occur if a DEX engine invariant is breached
    /// e.g. overflowing the position counter (`u16::MAX`)
    ///  or, overflowing the value circuit breaker (`u128::MAX`)
    ///
    /// In any of those cases, we do not want to allow a new position to be opened.
    #[tracing::instrument(level = "debug", skip_all)]
    async fn open_position(&mut self, position: position::Position) -> Result<()> {
        // Double-check that the position is in the `Opened` state
        if position.state != position::State::Opened {
            anyhow::bail!("attempted to open a position with a state besides `Opened`");
        }

        // Validate that the position ID doesn't collide
        if let Some(existing) = self.position_by_id(&position.id()).await? {
            anyhow::bail!(
                "attempted to open a position with ID {}, which already exists with state {:?}",
                position.id(),
                existing
            );
        }

        // Increase the position counter
        self.increment_position_counter(&position.phi.pair).await?;

        // Credit the DEX for the inflows from this position.
        self.vcb_credit(position.reserves_1()).await?;
        self.vcb_credit(position.reserves_2()).await?;

        // Finally, record the new position state.
        self.record_proto(event::position_open(&position));
        self.update_position(None, position).await?;

        Ok(())
    }

    /// Record execution against an opened position.
    ///
    /// The `context` parameter records the global context of the path in which
    /// the position execution happened. This may be completely different than
    /// the trading pair of the position itself, and is used to link the
    /// micro-scale execution (processed by this method) with the macro-scale
    /// context (a swap or arbitrage).
    #[tracing::instrument(level = "debug", skip_all)]
    async fn position_execution(
        &mut self,
        mut new_state: Position,
        context: DirectedTradingPair,
    ) -> Result<()> {
        let prev_state = self
            .position_by_id(&new_state.id())
            .await?
            .ok_or_else(|| anyhow::anyhow!("withdrew from unknown position {}", new_state.id()))?;

        anyhow::ensure!(
            matches!(&prev_state.state, position::State::Opened),
            "attempted to execute against a position with state {:?}, expected Opened",
            prev_state.state
        );
        anyhow::ensure!(
            matches!(&new_state.state, position::State::Opened),
            "supplied post-execution state {:?}, expected Opened",
            prev_state.state
        );

        // Handle "close-on-fill": automatically flip the position state to "closed" if
        // either of the reserves are zero.
        if new_state.close_on_fill {
            if new_state.reserves.r1 == 0u64.into() || new_state.reserves.r2 == 0u64.into() {
                tracing::debug!(
                    id = ?new_state.id(),
                    r1 = ?new_state.reserves.r1,
                    r2 = ?new_state.reserves.r2,
                    "marking position as closed due to close-on-fill"
                );
                new_state.state = position::State::Closed;
            }
        }

        // Optimization: it's possible that the position's reserves haven't
        // changed, and that we're about to do a no-op update. This can happen
        // when saving a frontier, for instance, since the FillRoute code saves
        // the entire frontier when it finishes.
        //
        // If so, skip the write, but more importantly, skip emitting an event,
        // so tooling doesn't get confused about a no-op execution.
        if prev_state != new_state {
            self.record_proto(event::position_execution(&prev_state, &new_state, context));
            self.update_position(Some(prev_state), new_state).await?;
        }

        Ok(())
    }

    /// Withdraw from a closed position, incrementing its sequence number.
    ///
    /// Updates the position's reserves and rewards to zero and returns the withdrawn balance.
    #[tracing::instrument(level = "debug", skip(self))]
    async fn withdraw_position(
        &mut self,
        position_id: position::Id,
        sequence: u64,
    ) -> Result<Balance> {
        let prev_state = self
            .position_by_id(&position_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("withdrew from unknown position {}", position_id))?;

        // Next, check that the withdrawal is consistent with the position state.
        // This should be redundant with the value balance mechanism (clients should
        // only be able to get the required input LPNFTs if the state transitions are
        // consistent), but we check it here for defense in depth.
        //
        // This is just a check that sequence == current_sequence + 1, with extra logic
        // so that we treat "closed" as "sequence -1".
        if sequence == 0 {
            if prev_state.state != position::State::Closed {
                anyhow::bail!(
                    "attempted to withdraw position {} with state {}, expected Closed",
                    position_id,
                    prev_state.state
                );
            }
        } else {
            if let position::State::Withdrawn {
                sequence: current_sequence,
            } = prev_state.state
            {
                if current_sequence + 1 != sequence {
                    anyhow::bail!(
                        "attempted to withdraw position {} with sequence {}, expected {}",
                        position_id,
                        sequence,
                        current_sequence + 1
                    );
                }
            } else {
                anyhow::bail!(
                    "attempted to withdraw position {} with state {}, expected Withdrawn",
                    position_id,
                    prev_state.state
                );
            }
        }

        // Record an event prior to updating the position state, so we have access to
        // the current reserves.
        self.record_proto(event::position_withdraw(position_id, &prev_state));

        // Grab a copy of the final reserves of the position to return to the caller.
        let reserves = prev_state.reserves.balance(&prev_state.phi.pair);

        // Debit the DEX for the outflows from this position.
        self.vcb_debit(prev_state.reserves_1()).await?;
        self.vcb_debit(prev_state.reserves_2()).await?;

        // Finally, update the position. This has two steps:
        // - update the state with the correct sequence number;
        // - zero out the reserves, to prevent double-withdrawals.
        let new_state = {
            let mut new_state = prev_state.clone();
            // We just checked that the supplied sequence number is incremented by 1 from prev.
            new_state.state = position::State::Withdrawn { sequence };
            new_state.reserves = Reserves::zero();
            new_state
        };

        self.update_position(Some(prev_state), new_state).await?;

        Ok(reserves)
    }
}

impl<T: StateWrite + ?Sized> PositionManager for T {}

#[async_trait]
pub(crate) trait Inner: StateWrite {
    /// Writes a position to the state, updating all necessary indexes.
    ///
    /// This should be the SOLE ENTRYPOINT for writing positions to the state.
    /// All other position changes exposed by the `PositionManager` should run through here.
    #[tracing::instrument(level = "debug", skip_all,  fields(id = ?new_state.id()))]
    async fn update_position(
        &mut self,
        prev_state: Option<Position>,
        new_state: Position,
    ) -> Result<()> {
        use position::State::*;

        tracing::debug!(?prev_state, ?new_state, "updating position state");

        let id = new_state.id();

        // Clear any existing indexes of the position, since changes to the
        // reserves or the position state might have invalidated them.
        if let Some(prev_state) = prev_state.as_ref() {
            self.deindex_position_by_price(&prev_state, &id);
        }

        // Only index the position's liquidity if it is active.
        if new_state.state == Opened {
            self.index_position_by_price(&new_state, &id);
        }

        if new_state.state == Closed {
            // Make sure that we don't double decrement the position
            // counter if a position was queued for closure AND closed
            // by the DEX engine.
            let is_already_closed = prev_state
                .as_ref()
                .map_or(false, |old_position| old_position.state == Closed);
            if !is_already_closed {
                self.decrement_position_counter(&new_state.phi.pair).await?;
            }
        }

        // Update the available liquidity for this position's trading pair.
        // TODO: refactor and streamline this method while implementing eviction.
        self.update_available_liquidity(&prev_state, &new_state)
            .await?;

        self.put(state_key::position_by_id(&id), new_state);
        Ok(())
    }

    fn index_position_by_price(&mut self, position: &position::Position, id: &position::Id) {
        let (pair, phi) = (position.phi.pair, &position.phi);
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

    fn deindex_position_by_price(&mut self, position: &Position, id: &position::Id) {
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
        prev_position: &Option<Position>,
        position: &Position,
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
}
impl<T: StateWrite + ?Sized> Inner for T {}
