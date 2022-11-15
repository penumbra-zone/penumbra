use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{action::ProposalSubmit, Transaction};
use tracing::instrument;

use crate::action_handler::ActionHandler;
use crate::governance::{check, execute, proposal::ProposalList};

#[async_trait]
impl ActionHandler for ProposalSubmit {
    #[instrument(name = "proposal_submit", skip(self, context))]
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        check::stateless::proposal_submit(self)
    }

    #[instrument(name = "proposal_submit", skip(self, state))]
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        let auth_hash = context.transaction_body().auth_hash();

        check::stateful::proposal_submit(&state, self).await
    }

    #[instrument(name = "proposal_submit", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        execute::proposal_submit(state, self).await;

        Ok(())
    }
}
