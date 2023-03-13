use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_crypto::{
    dex::{
        lp::position::{self, Position},
        DirectedTradingPair,
    },
    Value,
};
use penumbra_proto::{DomainType, StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

use super::state_key;

#[async_trait]
pub trait PositionRead: StateRead {
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
}

impl<T: StateWrite + ?Sized> PositionManager for T {}

#[async_trait]
trait Inner: StateWrite {
    fn index_position(&mut self, metadata: &position::Metadata) {
        let (pair, phi) = (metadata.position.phi.pair, &metadata.position.phi);
        let id_bytes = metadata.position.id().encode_to_vec();
        if metadata.reserves.r2 != 0u64.into() {
            // Index this position for trades FROM asset 1 TO asset 2, since the position has asset 2 to give out.
            let pair12 = DirectedTradingPair {
                start: pair.asset_1(),
                end: pair.asset_2(),
            };
            let phi12 = phi.component.clone();
            self.nonconsensus_put_raw(
                state_key::internal::price_index::key(&pair12, &phi12),
                id_bytes.clone(),
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
                state_key::internal::price_index::key(&pair21, &phi21),
                id_bytes,
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
        self.nonconsensus_delete(state_key::internal::price_index::key(&pair12, &phi12));
        self.nonconsensus_delete(state_key::internal::price_index::key(&pair21, &phi21));
    }
}
impl<T: StateWrite + ?Sized> Inner for T {}
