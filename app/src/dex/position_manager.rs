use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_crypto::{
    asset,
    dex::{
        lp::position::{self, Position},
        DirectedTradingPair,
    },
    fixpoint::U128x128,
    Amount, Value,
};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

use super::state_key;
use futures::Stream;
use futures::StreamExt;
use std::{
    collections::{BTreeMap, BTreeSet},
    iter::FromIterator,
    pin::Pin,
};

#[async_trait]
pub trait PositionRead: StateRead {
    /// Return a stream of all [`position::Metadata`] available.
    fn all_positions(&self) -> Pin<Box<dyn Stream<Item = Result<position::Position>> + Send + '_>> {
        let prefix = state_key::all_positions();
        self.prefix(prefix)
            .map(|entry| match entry {
                Ok((_, metadata)) => Ok(metadata),
                Err(e) => Err(e),
            })
            .boxed()
    }

    /// Return a stream of all [`position::Id`] available based on a starting asset.
    fn all_positions_from(
        &self,
        from: &asset::Id,
    ) -> Pin<Box<dyn Stream<Item = Result<position::Id>> + Send + '_>> {
        let prefix = state_key::internal::price_index::from_asset_prefix(from);
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

    /// Returns a stream of [`position::Id`] ordered by effective price.
    fn positions_by_price(
        &self,
        pair: &DirectedTradingPair,
    ) -> Pin<Box<dyn Stream<Item = Result<position::Id>> + Send + 'static>> {
        let prefix = state_key::internal::price_index::prefix(pair);
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
}
impl<T: StateRead + ?Sized> PositionRead for T {}

/// Manages liquidity positions within the chain state.
#[async_trait]
pub trait PositionManager: StateWrite + PositionRead {
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

    /// Fill a trade of `input` value against a specific position `id`, writing
    /// the updated reserves to the chain state and returning a pair of `(unfilled, output)`.
    async fn fill_against(&mut self, input: Value, id: &position::Id) -> Result<(Value, Value)> {
        let mut position = self
            .position_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("tried to fill against unknown position {id:?}"))?;

        tracing::debug!(?input, ?position, "executing against position");

        if position.state != position::State::Opened {
            return Err(anyhow::anyhow!(
                "tried to fill against non-Opened position {:?}",
                id
            ));
        }

        let (unfilled, new_reserves, output) = position
            .phi
            .fill(input, &position.reserves)
            .context(format!(
                "could not fill {:?} against position {:?}",
                input, id
            ))?;

        position.reserves = new_reserves;
        self.put_position(position);

        Ok((unfilled, output))
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
        from: asset::Id,
        fixed_candidates: Vec<asset::Id>,
    ) -> Result<Vec<asset::Id>> {
        // query the state for liquidity-based candidates
        let mut position_ids = self.all_positions_from(&from);

        let mut positions = BTreeSet::from_iter(fixed_candidates.iter().cloned());

        // Bucket all positions "from" this asset and order by available liquidity of the "to" asset.
        let mut buckets = BTreeMap::new();

        // TODO: would it be more efficient to index trading pairs by liquidity somewhere in non-consensus storage?
        while let Some(id) = position_ids.next().await {
            let id = &id?;
            let position = self
                .position_by_id(&id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("position id {:?} not found in state", id))?;

            let to = if position.phi.pair.asset_1() == from {
                position.phi.pair.asset_2()
            } else {
                position.phi.pair.asset_1()
            };

            // Increment the total reserves for the "to" asset in the bucket.
            let position_reserves = position.reserves_for(to).ok_or_else(|| {
                anyhow::anyhow!(
                    "position {:?} does not contain reserves for asset {:?}",
                    id,
                    to
                )
            })?;
            buckets
                .entry(to)
                .and_modify(|reserves| *reserves = *reserves + position_reserves)
                .or_insert(position_reserves);
        }

        // Now we have buckets corresponding to the "to" assets directly routable from the "from" asset, containing liquidity.
        // Sort by liquidity:
        let mut v = Vec::from_iter(buckets);
        v.sort_by(|&(_, a), &(_, b)| b.cmp(&a));

        // Add the top buckets until we reach the maximum number of candidates.
        for (to, _) in v {
            // TODO: make this size a chain parameter?
            if positions.len() >= 5 {
                break;
            }
            positions.insert(to);
        }

        Ok(positions.into_iter().collect())
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
            tracing::debug!(pair = ?pair12, ?id, "indexing position 12");
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
            tracing::debug!(pair = ?pair21, ?id, "indexing position 21");
        }
    }

    fn deindex_position(&mut self, position: &Position) {
        let id = position.id();
        tracing::debug!(?id, "deindexing position");
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
