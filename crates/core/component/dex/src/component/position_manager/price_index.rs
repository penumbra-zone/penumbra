use cnidarium::StateWrite;
use penumbra_sdk_proto::StateWriteProto;

use crate::{
    lp::position::{self, Position},
    state_key::engine,
    DirectedTradingPair,
};

use anyhow::Result;
use position::State::*;

pub(crate) trait PositionByPriceIndex: StateWrite {
    fn update_position_by_price_index(
        &mut self,
        position_id: &position::Id,
        prev_state: &Option<Position>,
        new_state: &Position,
    ) -> Result<()> {
        // Clear an existing record for the position, since changes to the
        // reserves or the position state might have invalidated it.
        if let Some(prev_lp) = prev_state {
            self.deindex_position_by_price(prev_lp, position_id);
        }

        if matches!(new_state.state, Opened) {
            self.index_position_by_price(new_state, position_id);
        }

        Ok(())
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
        self.nonverifiable_delete(engine::price_index::key(&pair12, &phi12, &id));
        self.nonverifiable_delete(engine::price_index::key(&pair21, &phi21, &id));
    }
}
impl<T: StateWrite + ?Sized> PositionByPriceIndex for T {}

trait Inner: StateWrite {
    fn index_position_by_price(&mut self, position: &position::Position, id: &position::Id) {
        let (pair, phi) = (position.phi.pair, &position.phi);
        if position.reserves.r2 != 0u64.into() {
            // Index this position for trades FROM asset 1 TO asset 2, since the position has asset 2 to give out.
            let pair12 = DirectedTradingPair {
                start: pair.asset_1(),
                end: pair.asset_2(),
            };
            let phi12 = phi.component.clone();
            self.nonverifiable_put(
                engine::price_index::key(&pair12, &phi12, &id),
                position.clone(),
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
            self.nonverifiable_put(
                engine::price_index::key(&pair21, &phi21, &id),
                position.clone(),
            );
            tracing::debug!("indexing position for 2=>1 trades");
        }
    }
}

impl<T: StateWrite + ?Sized> Inner for T {}
