use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::PositionClose, Transaction};

use crate::action_handler::ActionHandler;
use crate::dex::PositionManager;

#[async_trait]
/// Debits an opened position NFT and credits a closed position NFT.
impl ActionHandler for PositionClose {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        // Nothing to do: the only validation is of the state change,
        // and that's done by the value balance mechanism.
        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // TODO: how much logic should live in ActionHandlers vs PositionManager?
        state.position_close(&self.position_id).await
    }
}
