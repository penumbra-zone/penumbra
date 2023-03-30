use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::Ics20Withdrawal, Transaction};

use crate::action_handler::ActionHandler;
use crate::ibc::transfer::Ics20TransferReadExt as _;
use crate::ibc::transfer::Ics20TransferWriteExt as _;

#[async_trait]
impl ActionHandler for Ics20Withdrawal {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        self.validate()
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        state.withdrawal_check(self).await
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.withdrawal_execute(self).await;

        Ok(())
    }
}
