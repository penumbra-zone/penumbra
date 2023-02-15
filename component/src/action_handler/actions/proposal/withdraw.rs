use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::ProposalWithdraw, Transaction};
use tracing::instrument;

use crate::{
    action_handler::ActionHandler,
    governance::{check, execute},
};

#[async_trait]
impl ActionHandler for ProposalWithdraw {
    #[instrument(name = "proposal_withdraw", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        check::stateless::proposal_withdraw(self)
    }

    #[instrument(name = "proposal_withdraw", skip(self, state))]
    async fn check_stateful<S: StateRead>(&self, state: Arc<S>) -> Result<()> {
        check::stateful::proposal_withdraw(&state, self).await
    }

    #[instrument(name = "proposal_withdraw", skip(self, state))]
    async fn execute<S: StateWrite>(&self, state: S) -> Result<()> {
        execute::proposal_withdraw(state, self).await
    }
}
