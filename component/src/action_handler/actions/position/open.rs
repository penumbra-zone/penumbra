use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::PositionOpen, Transaction};
use tracing::instrument;

use crate::action_handler::ActionHandler;

#[async_trait]
/// Debits the initial reserves and credits an opened position NFT.
impl ActionHandler for PositionOpen {
    #[instrument(name = "position_open", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        // It's important to reject all LP actions for now, to prevent
        // inflation / minting bugs until we implement all required checks
        // (e.g., minting tokens by withdrawing reserves we don't check)
        Err(anyhow::anyhow!("lp actions not supported yet"))
    }

    #[instrument(name = "position_open", skip(self, _state))]
    async fn check_stateful<S: StateRead>(&self, _state: Arc<S>) -> Result<()> {
        // It's important to reject all LP actions for now, to prevent
        // inflation / minting bugs until we implement all required checks
        // (e.g., minting tokens by withdrawing reserves we don't check)
        Err(anyhow::anyhow!("lp actions not supported yet"))
    }

    #[instrument(name = "position_open", skip(self, _state))]
    async fn execute<S: StateWrite>(&self, _state: S) -> Result<()> {
        // let position = self.position;
        // let initial_reserves = self.initial_reserves;
        // let lpnft = state.position_open(position, initial_reserves).await?;
        // TODO: implement

        Ok(())
    }
}
