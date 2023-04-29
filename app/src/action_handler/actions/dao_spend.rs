use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::action::DaoSpend;

use crate::{dao::view::StateWriteExt, ActionHandler};

#[async_trait]
impl ActionHandler for DaoSpend {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // We can't statelessly check that the DAO has enough funds to spend, because we don't know
        // what its state is here.
        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // Instead of checking here, we just check during execution, which will fail if we try to
        // overdraw the DAO.
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // This will fail if we try to overdraw the DAO, so we can never spend more than we have.
        state.dao_withdraw(self.value).await
    }
}
