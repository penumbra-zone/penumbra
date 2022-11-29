use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
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
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        let auth_hash = context.transaction_body().auth_hash();

        check::stateful::proposal_withdraw(&state, &auth_hash, self).await
    }

    #[instrument(name = "proposal_withdraw", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        execute::proposal_withdraw(state, self).await
    }
}
