use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::Transaction;

use super::ActionHandler;

#[async_trait]
impl ActionHandler for Transaction {
    fn check_tx_stateless(&self) -> anyhow::Result<()> {
        for action in self.actions() {
            action.check_tx_stateless()?;
        }

        Ok(())
    }

    async fn check_tx_stateful(&self, state: Arc<State>) -> Result<()> {
        for action in self.actions() {
            action.check_tx_stateful(state.clone()).await?;
        }

        Ok(())
    }

    async fn execute_tx(&self, state: &mut StateTransaction) -> Result<()> {
        for action in self.actions() {
            action.execute_tx(state).await?;
        }

        Ok(())
    }
}
