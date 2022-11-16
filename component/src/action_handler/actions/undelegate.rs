use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{action::Undelegate, Transaction};
use tracing::instrument;

use crate::action_handler::ActionHandler;

#[async_trait]
impl ActionHandler for Undelegate {
    #[instrument(name = "undelegate", skip(self, context))]
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        // All stateless undelegation-related checks are performed
        // at the Transaction-level.
        Ok(())
    }

    #[instrument(name = "undelegate", skip(self, state, context))]
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        todo!()
    }

    #[instrument(name = "undelegate", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        todo!()
    }
}
