use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::ActionHandler;
use penumbra_proto::StateWriteProto as _;

use crate::{
    component::{PositionManager, PositionRead},
    event,
    lp::{action::PositionOpen, position},
};

#[async_trait]
/// Debits the initial reserves and credits an opened position NFT.
impl ActionHandler for PositionOpen {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // TODO(chris, erwan, henry): brainstorm safety on `TradingFunction`.
        // Check:
        //  + reserves are at most 80 bits wide,
        //  + the trading function coefficients are at most 80 bits wide.
        //  + at least some assets are provisioned.
        //  + the trading function coefficients are non-zero,
        //  + the trading function doesn't specify a cyclic pair,
        //  + the fee is <=50%.
        self.position.check_stateless()?;
        if self.position.state != position::State::Opened {
            anyhow::bail!("attempted to open a position with a state besides `Opened`");
        }
        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // Validate that the position ID doesn't collide
        state.check_position_id_unused(&self.position.id()).await?;
        state.put_position(self.position.clone()).await?;
        state.record_proto(event::position_open(self));
        Ok(())
    }
}
