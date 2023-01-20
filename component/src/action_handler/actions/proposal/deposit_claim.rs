use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{action::ProposalDepositClaim, Transaction};
use tracing::instrument;

use crate::action_handler::ActionHandler;
use crate::governance::{check, execute};

#[async_trait]
impl ActionHandler for ProposalDepositClaim {
    #[instrument(name = "proposal_deposit_claim", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        check::stateless::proposal_deposit_claim(self)
    }

    #[instrument(name = "proposal_deposit_claim", skip(self, state))]
    async fn check_stateful(&self, state: Arc<State>) -> Result<()> {
        check::stateful::proposal_deposit_claim(&state, self).await
    }

    #[instrument(name = "proposal_deposit_claim", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        execute::proposal_deposit_claim(state, self).await?;

        Ok(())
    }
}
