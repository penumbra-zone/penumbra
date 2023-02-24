use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::ProposalWithdraw, Transaction};
use tracing::instrument;

use crate::{
    action_handler::ActionHandler,
    governance::{execute, StateReadExt},
};

#[async_trait]
impl ActionHandler for ProposalWithdraw {
    #[instrument(name = "proposal_withdraw", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        // Enforce a maximum length on proposal withdrawal reasons; 80 characters seems reasonable.
        const PROPOSAL_WITHDRAWAL_REASON_LIMIT: usize = 80;

        if self.reason.len() > PROPOSAL_WITHDRAWAL_REASON_LIMIT {
            return Err(anyhow::anyhow!(
                "proposal withdrawal reason must fit within {PROPOSAL_WITHDRAWAL_REASON_LIMIT} characters"
            ));
        }

        Ok(())
    }

    #[instrument(name = "proposal_withdraw", skip(self, state))]
    async fn check_stateful<S: StateRead>(&self, state: Arc<S>) -> Result<()> {
        // Any voteable proposal can be withdrawn
        state.check_proposal_voteable(self.proposal).await?;
        Ok(())
    }

    #[instrument(name = "proposal_withdraw", skip(self, state))]
    async fn execute<S: StateWrite>(&self, state: S) -> Result<()> {
        execute::proposal_withdraw(state, self).await
    }
}
