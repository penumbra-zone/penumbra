use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::PositionOpen, Transaction};

use crate::action_handler::ActionHandler;
use crate::dex::{PositionManager, PositionRead};

#[async_trait]
/// Debits the initial reserves and credits an opened position NFT.
impl ActionHandler for PositionOpen {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        // Check:
        //  + reserves are at most 112 bits wide,
        //  + at least some assets are provisioned.
        self.position.reserves.check_bounds()?;
        // Check:
        //  + the trading function coefficients are at most 112 bits wide.
        //  + the trading function coefficients are non-zero,
        //  + the trading function doesn't specify a cyclic pair,
        //  + the fee is <=50%.
        self.position.check_stateless()?;

        // TODO: any other checks of the trading function that should be performed?

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        // Validate that the position ID doesn't collide
        state.check_position_id_unused(&self.position.id()).await?;

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // Write the newly opened position.
        state.put_position(self.position.clone());

        Ok(())
    }
}
