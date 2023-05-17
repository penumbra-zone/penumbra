use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_crypto::{
    asset,
    dex::{
        lp::position::{self, Position},
        DirectedTradingPair,
    },
    Value,
};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{EscapedByteSlice, StateRead, StateWrite};

use super::state_key;
use futures::Stream;
use futures::StreamExt;
use std::{collections::BTreeSet, iter::FromIterator, pin::Pin, sync::Arc};

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

    async fn worst_position(
        &self,
        pair: &DirectedTradingPair,
    ) -> Result<Option<position::Position>> {
        // Since the other direction might not have any positions, we need to
        // fetch the last one in the index.
        //
        // TODO: Maybe we should have a separate index for this?
        let positions_by_price = self.positions_by_price(pair);
        let positions = positions_by_price.collect::<Vec<_>>().await;

        let id = match positions.last() {
            Some(id) => match id {
                Ok(id) => id.clone(),
                Err(e) => {
                    return Err(anyhow::anyhow!("{}", e).context("failed to fetch worst position"));
                }
            },
            None => return Ok(None),
        };
        self.position_by_id(&id).await
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

            assert_eq!(
                position.state,
                position::State::Opened,
                "value balance mechanism should have ensured position is Opened"
            );

            position.state = position::State::Closed;
            self.put_position(position);
        }
        self.object_delete(state_key::pending_position_closures());
        Ok(())
    }

    /// Writes a position to the state, updating all necessary indexes.
    #[tracing::instrument(level = "debug", skip(self, position), fields(id = ?position.id()))]
    fn put_position(&mut self, position: position::Position) {
        let id = position.id();
        tracing::debug!(?position);
        // Clear any existing indexes of the position, since changes to the
        // reserves or the position state might have invalidated them.
        self.deindex_position(&position);
        // Only index the position's liquidity if it is active.
        if position.state == position::State::Opened {
            self.index_position(&position);
        }
        self.put(state_key::position_by_id(&id), position);
    }

    /// Fill a trade of `input` value against all available positions, until completion, or
    /// the available liquidity is exhausted.
    ///
    /// Returns the unfilled amount, the total output of the trade, and the ids of positions
    /// that were executed against.
    ///
    /// TODO(erwan): global slippage parameter should act as a "fail-early" guard here, but we'd
    /// need to get some signal about the phi of the position we're executing against.
    async fn fill(
        &mut self,
        input: Value,
        pair: DirectedTradingPair,
    ) -> Result<(Value, Value, Vec<position::Id>)> {
        let mut position_ids = self.positions_by_price(&pair);

        let zero = Value {
            asset_id: input.asset_id,
            amount: 0u64.into(),
        };

        let mut remaining = input;
        let mut total_output = zero;

        let mut positions = vec![];

        while let Some(id) = position_ids.next().await {
            if remaining == zero {
                break;
            }

            let id = &id?;
            positions.push(id.clone());
            let (unfilled, output) = self.fill_against(remaining, id).await?;
            remaining = unfilled;
            total_output = Value {
                asset_id: input.asset_id,
                amount: total_output.amount + output.amount,
            };
        }

        Ok((remaining, total_output, positions))
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

        // TODO: fill in an implementation that does dynamic candidate
        // selection, but without scanning every single position in every graph
        // traversal iteration.
        //
        // Perhaps the per-asset candidate sets should actually be computed once
        // per block and stored as a BTreeMap in the object store?

        Ok(candidates.into_iter().collect())
    }
}

impl<T: StateWrite + ?Sized> PositionManager for T {}

#[async_trait]
pub(super) trait Inner: StateWrite {
    fn index_position(&mut self, position: &position::Position) {
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

    fn deindex_position(&mut self, position: &Position) {
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
