use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::action::DaoDeposit;

use crate::dao::view::StateWriteExt;
use crate::ActionHandler;

#[async_trait]
impl ActionHandler for DaoDeposit {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Any deposit into the DAO is valid.
        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // Any deposit into the DAO is valid.
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.dao_deposit(self.value).await
    }
}
