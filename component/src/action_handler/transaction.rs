use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::Transaction;

use crate::shielded_pool::consensus_rules;

use super::ActionHandler;

#[async_trait]
impl ActionHandler for Transaction {
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?
        let auth_hash = context.transaction_body().auth_hash();

        // 1. Check binding signature.
        self.binding_verification_key()
            .verify(auth_hash.as_ref(), self.binding_sig())
            .context("binding signature failed to verify")?;

        // TODO: these can all be parallel tasks
        for action in self.actions() {
            action.check_stateless(context.clone())?;
        }

        consensus_rules::stateless::num_clues_equal_to_num_outputs(self)?;
        consensus_rules::stateless::check_memo_exists_if_outputs_absent_if_not(self)?;

        Ok(())
    }

    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        // TODO: these can all be parallel tasks
        for action in self.actions() {
            action
                .check_stateful(state.clone(), context.clone())
                .await?;
        }

        Ok(())
    }

    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        for action in self.actions() {
            action.execute(state).await?;
        }

        Ok(())
    }
}
