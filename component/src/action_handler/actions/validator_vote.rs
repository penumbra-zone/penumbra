use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{action::ValidatorVote, Transaction};
use tracing::instrument;

use crate::{
    action_handler::ActionHandler,
    governance::{check, execute},
};

#[async_trait]
impl ActionHandler for ValidatorVote {
    #[instrument(name = "validator_vote", skip(self, context))]
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        check::stateless::validator_vote(self)
    }

    #[instrument(name = "validator_vote", skip(self, state, context))]
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        check::stateful::validator_vote(&state, self).await
    }

    #[instrument(name = "validator_vote", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        execute::validator_vote(state, self).await
    }
}
