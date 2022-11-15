use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{action::Output, Transaction};
use tracing::instrument;

use crate::action_handler::ActionHandler;

#[async_trait]
impl ActionHandler for Output {
    #[instrument(name = "output", skip(self, context))]
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        todo!()
    }

    #[instrument(name = "output", skip(self, state))]
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        todo!()
    }

    #[instrument(name = "output", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        todo!()
    }
}
