use anyhow::Result;
use cnidarium::StateWrite;
use penumbra_sdk_num::Amount;
use position::State::*;
use tracing::instrument;

use crate::lp::position::{self, Position};
use crate::state_key::engine;
use crate::DirectedTradingPair;
use async_trait::async_trait;
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};

#[async_trait]
pub(crate) trait PositionVolumeTracker: StateWrite {
    async fn increase_volume_index(
        &mut self,
        id: &position::Id,
        prev_state: &Option<Position>,
        new_state: &Position,
    ) -> Result<()> {
        unimplemented!("increase_volume_index")
    }
}

impl<T: StateWrite + ?Sized> PositionVolumeTracker for T {}

trait Inner: StateWrite {
    #[instrument(skip(self))]
    async fn update_volume(
        &mut self,
        id: &position::Id,
        pair: DirectedTradingPair,
        old_volume: Amount,
        new_volume: Amount,
    ) -> Result<()> {
        Ok(())
    }
}

impl<T: StateWrite + ?Sized> Inner for T {}
