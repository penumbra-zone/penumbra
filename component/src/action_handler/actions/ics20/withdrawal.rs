use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::Ics20Withdrawal, Transaction};
use tracing::instrument;

use crate::action_handler::ActionHandler;
use crate::ibc::transfer::Ics20TransferReadExt as _;
use crate::ibc::transfer::Ics20TransferWriteExt as _;

#[async_trait]
impl ActionHandler for Ics20Withdrawal {
    #[instrument(name = "ics20_withdrawal", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        self.validate()
    }

    #[instrument(name = "ics20_withdrawal", skip(self, state))]
    async fn check_stateful<S: StateRead>(&self, state: Arc<S>) -> Result<()> {
        state.withdrawal_check(self).await
    }

    #[instrument(name = "ics20_withdrawal", skip(self, state))]
    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.withdrawal_execute(self).await;

        Ok(())
    }
}
