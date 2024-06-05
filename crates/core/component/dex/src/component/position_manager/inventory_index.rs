use cnidarium::StateWrite;
use tracing::instrument;

use crate::{
    lp::position::{self, Position},
    state_key::eviction_queue,
    DirectedTradingPair,
};

use anyhow::Result;
use position::State::*;

pub(super) trait PositionByInventoryIndex: StateWrite {
    fn update_position_by_inventory_index(
        &mut self,
        position_id: &position::Id,
        prev_state: &Option<Position>,
        new_state: &Position,
    ) -> Result<()> {
        // Clear an existing record of the position, since changes to the
        // reserves or the position state might have invalidated it.
        if let Some(prev_lp) = prev_state {
            self.deindex_position_by_inventory(prev_lp, position_id);
        }

        if matches!(new_state.state, Opened) {
            self.index_position_by_inventory(new_state, position_id);
        }

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> PositionByInventoryIndex for T {}

trait Inner: StateWrite {
    #[instrument(skip(self, position))]
    fn index_position_by_inventory(&mut self, position: &position::Position, id: &position::Id) {
        tracing::trace!("indexing position by inventory");
        let canonical_pair = position.phi.pair;
        // A position is bound to an unordered trading pair: A <> B.
        // We want to index the position by inventory for each direction:
        // A -> B
        let pair_ab = DirectedTradingPair::new(canonical_pair.asset_1(), canonical_pair.asset_2());
        let inventory_a = position
            .reserves_for(pair_ab.start)
            .expect("the directed trading pair is correct");
        let key_ab = eviction_queue::inventory_index::key(&pair_ab, inventory_a, id).to_vec();
        self.nonverifiable_put_raw(key_ab, vec![]);

        // B -> A
        let pair_ba = pair_ab.flip();
        let inventory_b = position
            .reserves_for(pair_ba.start)
            .expect("the directed trading pair is correct");
        let key_ba = eviction_queue::inventory_index::key(&pair_ba, inventory_b, id).to_vec();
        self.nonverifiable_put_raw(key_ba, vec![]);
    }

    fn deindex_position_by_inventory(
        &mut self,
        prev_position: &position::Position,
        id: &position::Id,
    ) {
        let canonical_pair = prev_position.phi.pair;

        // To deindex the position, we need to reconstruct the tuple of keys
        // that correspond to each direction of the trading pair:
        // A -> B
        let pair_ab = DirectedTradingPair::new(canonical_pair.asset_1(), canonical_pair.asset_2());
        let inventory_a = prev_position
            .reserves_for(pair_ab.start)
            .expect("the directed trading pair is correct");
        let key_ab = eviction_queue::inventory_index::key(&pair_ab, inventory_a, id).to_vec();
        self.nonverifiable_delete(key_ab);

        // B -> A
        let pair_ba = pair_ab.flip();
        let inventory_b = prev_position
            .reserves_for(pair_ba.start)
            .expect("the directed trading pair is correct");
        let key_ba = eviction_queue::inventory_index::key(&pair_ba, inventory_b, id).to_vec();
        self.nonverifiable_delete(key_ba);
    }
}
impl<T: StateWrite + ?Sized> Inner for T {}
