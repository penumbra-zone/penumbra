use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::ProposalDepositClaim, Transaction};
use tracing::instrument;

use crate::action_handler::ActionHandler;
use crate::governance::{execute, StateReadExt as _};

#[async_trait]
impl ActionHandler for ProposalDepositClaim {
    #[instrument(name = "proposal_deposit_claim", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        // No stateless checks are required for this action (all checks require state access)
        Ok(())
    }

    #[instrument(name = "proposal_deposit_claim", skip(self, state))]
    async fn check_stateful<S: StateRead>(&self, state: Arc<S>) -> Result<()> {
        // Any finished proposal can have its deposit claimed
        state.check_proposal_claimable(self.proposal).await?;
        // Check that the deposit amount matches the proposal being claimed
        state
            .check_proposal_claim_valid_deposit(self.proposal, self.deposit_amount)
            .await?;
        Ok(())
    }

    #[instrument(name = "proposal_deposit_claim", skip(self, state))]
    async fn execute<S: StateWrite>(&self, state: S) -> Result<()> {
        execute::proposal_deposit_claim(state, self).await?;

        Ok(())
    }
}
