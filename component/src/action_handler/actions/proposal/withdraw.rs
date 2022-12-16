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

    #[instrument(name = "proposal_withdraw", skip(self, _state))]
    async fn check_stateful(&self, _state: Arc<State>) -> Result<()> {
        // Disabled since this doesn't fit the shape of the new trait,
        // but not fixed because we want to change the proposal withdrawal
        // mechanism anyways.
        /*
        let effect_hash = context.transaction_body().effect_hash();

        check::stateful::proposal_withdraw(&state, &effect_hash, self).await
        */
        Ok(())
    }

    #[instrument(name = "proposal_withdraw", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        execute::proposal_withdraw(state, self).await
    }
}
