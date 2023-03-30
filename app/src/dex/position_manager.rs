use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_crypto::{
    dex::{
        lp::position::{self, Position},
        DirectedTradingPair,
    },
    Value,
};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

use super::state_key;
use futures::Stream;
use futures::StreamExt;
use std::pin::Pin;

#[async_trait]
pub trait PositionRead: StateRead {
    /// Return a stream of all [`position::Metadata`] available.
    fn all_positions(&self) -> Pin<Box<dyn Stream<Item = Result<position::Metadata>> + Send + '_>> {
        let prefix = state_key::all_positions();
        self.prefix(prefix)
            .map(|entry| match entry {
                Ok((_, metadata)) => Ok(metadata),
                Err(e) => Err(e),
            })
            .boxed()
    }

    /// Returns a stream of [`position::Id`] ordered by effective price.
    async fn positions_by_price(
        &self,
        pair: DirectedTradingPair,
    ) -> Pin<Box<dyn Stream<Item = Result<position::Id>> + Send + 'static>> {
        let prefix = state_key::internal::price_index::prefix(&pair);
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

    async fn position_by_id(&self, id: &position::Id) -> Result<Option<position::Metadata>> {
        self.get(&state_key::position_by_id(id)).await
    }

    async fn check_position_id_unused(&self, id: &position::Id) -> Result<()> {
        match self.get_raw(&state_key::position_by_id(id)).await? {
            Some(_) => Err(anyhow::anyhow!("position id {:?} already used", id)),
            None => Ok(()),
        }
    }
}
impl<T: StateRead + ?Sized> PositionRead for T {}

/// Manages liquidity positions within the chain state.
#[async_trait]
pub trait PositionManager: StateWrite + PositionRead {
    /// Writes a position to the state, updating all necessary indexes.
    fn put_position(&mut self, metadata: position::Metadata) {
        let id = metadata.position.id();
        // Clear any existing indexes of the position, since changes to the
        // reserves or the position state might have invalidated them.
        self.deindex_position(&metadata.position);
        // Only index the position's liquidity if it is active.
        if metadata.state == position::State::Opened {
            self.index_position(&metadata);
        }
        self.put(state_key::position_by_id(&id), metadata);
    }

    /// Fill a trade of `input` value against a specific position `id`, writing
    /// the updated reserves to the chain state and returning a pair of `(unfilled, output)`.
    async fn fill_against(&mut self, input: Value, id: &position::Id) -> Result<(Value, Value)> {
        let mut metadata = self
            .position_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("tried to fill against unknown position {:?}", id))?;

        if metadata.state != position::State::Opened {
            return Err(anyhow::anyhow!(
                "tried to fill against non-Opened position {:?}",
                id
            ));
        }

        let (unfilled, new_reserves, output) = metadata
            .position
            .phi
            .fill(input, &metadata.reserves)
            .context(format!(
                "could not fill {:?} against position {:?}",
                input, id
            ))?;

        metadata.reserves = new_reserves;
        self.put_position(metadata);

        Ok((unfilled, output))
    }

    /// Fill a trade of `input` value against all available positions, until completion, or
    /// the available liquidity is exhausted. Returns a tuple containing the unfilled amount,
    /// and the total output of the trade.
    /// TODO(erwan): global slippage parameter should act as a "fail-early" guard here, but we'd
    /// need to get some signal about the phi of the position we're executing against.
    async fn fill(&mut self, input: Value, pair: DirectedTradingPair) -> Result<(Value, Value)> {
        let mut position_ids = self.positions_by_price(pair).await;

        let zero = Value {
            asset_id: input.asset_id,
            amount: 0u64.into(),
        };

        let mut remaining = input;
        let mut total_output = zero;

        while let Some(id) = position_ids.next().await {
            if remaining == zero {
                break;
            }
            let (unfilled, output) = self.fill_against(remaining, &id?).await?;
            remaining = unfilled;
            total_output = Value {
                asset_id: input.asset_id,
                amount: total_output.amount + output.amount,
            };
        }

        Ok((remaining, total_output))
    }
}

impl<T: StateWrite + ?Sized> PositionManager for T {}

#[async_trait]
trait Inner: StateWrite {
    fn index_position(&mut self, metadata: &position::Metadata) {
        let (pair, phi) = (metadata.position.phi.pair, &metadata.position.phi);
        let id = metadata.position.id();
        if metadata.reserves.r2 != 0u64.into() {
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
        }
        if metadata.reserves.r1 != 0u64.into() {
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
        }
    }

    fn deindex_position(&mut self, position: &Position) {
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
        self.nonconsensus_delete(state_key::internal::price_index::key(
            &pair12,
            &phi12,
            &position.id(),
        ));
        self.nonconsensus_delete(state_key::internal::price_index::key(
            &pair21,
            &phi21,
            &position.id(),
        ));
    }
}
impl<T: StateWrite + ?Sized> Inner for T {}
