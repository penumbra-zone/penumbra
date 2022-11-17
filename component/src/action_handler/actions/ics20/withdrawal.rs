use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{action::Ics20Withdrawal, Transaction};
use tracing::instrument;

use crate::action_handler::ActionHandler;

#[async_trait]
impl ActionHandler for Ics20Withdrawal {
    #[instrument(name = "ics20_withdrawal", skip(self, context))]
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        self.validate()
    }

    #[instrument(name = "ics20_withdrawal", skip(self, state))]
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        <penumbra_storage::State as crate::ibc::transfer::Ics20TransferReadExt>::withdrawal_check(
            state.clone(),
            self,
        )
        .await
    }

    #[instrument(name = "ics20_withdrawal", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        <&mut penumbra_storage::StateTransaction<'_> as crate::ibc::transfer::Ics20TransferWriteExt>::withdrawal_execute(state, self).await;

        Ok(())
    }
}
