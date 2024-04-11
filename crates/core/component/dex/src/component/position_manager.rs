use std::future;
use std::{pin::Pin, sync::Arc};

use anyhow::{bail, ensure, Result};
use async_stream::try_stream;
use async_trait::async_trait;
use cnidarium::{EscapedByteSlice, StateRead, StateWrite};
use futures::Stream;
use futures::StreamExt;
use penumbra_asset::{asset, Balance};
use penumbra_proto::DomainType;
use penumbra_proto::{StateReadProto, StateWriteProto};

use crate::component::position_manager::{
    base_liquidity_index::AssetByLiquidityIndex, inventory_index::PositionByInventoryIndex,
    price_index::PositionByPriceIndex,
};
use crate::lp::Reserves;
use crate::{
    component::position_manager::counter::PositionCounter,
    component::ValueCircuitBreaker,
    lp::position::{self, Position},
    state_key::engine,
    DirectedTradingPair,
};
use crate::{event, state_key};

const DYNAMIC_ASSET_LIMIT: usize = 10;

pub(crate) mod base_liquidity_index;
pub(crate) mod counter;
pub(crate) mod inventory_index;
pub(crate) mod price_index;

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
        let prefix = engine::price_index::prefix(pair);
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
        start: &asset::Id,
    ) -> Pin<Box<dyn Stream<Item = Result<asset::Id>> + Send + 'static>> {
        let prefix = engine::routable_assets::starting_from(start);
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

        // Optimization: skip state update if the position is already closed.
        // This can happen if the position was queued for closure and premptively
        // closed by the DEX engine during execution (e.g. auto-closing).
        if prev_state.state == position::State::Closed {
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
trait Inner: StateWrite {
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
        tracing::debug!(?prev_state, ?new_state, "updating position state");

        let id = new_state.id();

        // Assert `update_position` state transitions invariants:
        Self::guard_invalid_transitions(&prev_state, &new_state, &id)?;

        // Update the DEX engine indices:
        self.update_position_by_price_index(&prev_state, &new_state, &id)?;
        self.update_position_by_inventory_index(&prev_state, &new_state, &id)?;
        self.update_asset_by_base_liquidity_index(&prev_state, &new_state, &id)
            .await?;
        self.update_trading_pair_position_counter(&prev_state, &new_state, &id)
            .await?;

        self.put(state_key::position_by_id(&id), new_state);
        Ok(())
    }

    fn guard_invalid_transitions(
        prev_state: &Option<Position>,
        new_state: &Position,
        id: &position::Id,
    ) -> Result<()> {
        use position::State::*;

        if let Some(prev_lp) = prev_state {
            tracing::debug!(?id, prev = ?prev_lp.state, new = ?new_state.state, "evaluating state transition");
            match (prev_lp.state, new_state.state) {
                (Opened, Opened) => {}
                (Opened, Closed) => {}
                (Closed, Closed) => { /* no-op but allowed */ }
                (Closed, Withdrawn { sequence }) => {
                    ensure!(
                        sequence == 0,
                        "withdrawn positions must have their sequence start at zero (found: {})",
                        sequence
                    );
                }
                (Withdrawn { sequence: old_seq }, Withdrawn { sequence: new_seq }) => {
                    let expected_seq = old_seq.saturating_add(1);
                    ensure!(
                        new_seq == expected_seq,
                        "withdrawn must increase 1-by-1 (old: {}, new: {}, expected: {})",
                        old_seq,
                        new_seq,
                        expected_seq
                    );
                }
                _ => bail!("invalid transition"),
            }
        } else {
            ensure!(
                matches!(new_state.state, Opened),
                "fresh positions MUST start in the `Opened` state (found: {:?})",
                new_state.state
            );
        }

        Ok(())
    }
}
impl<T: StateWrite + ?Sized> Inner for T {}
