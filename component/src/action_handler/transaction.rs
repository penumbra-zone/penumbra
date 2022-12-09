use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::NoteSource;
use penumbra_storage::{State, StateTransaction, StateWrite as _};
use penumbra_transaction::Transaction;

use crate::shielded_pool::consensus_rules;

use self::stateful::{claimed_anchor_is_valid, fmd_parameters_valid};

use super::ActionHandler;

mod stateful;
mod stateless;

use stateless::{no_duplicate_nullifiers, valid_binding_signature};

#[async_trait]
impl ActionHandler for Transaction {
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?

        valid_binding_signature(self)?;
        no_duplicate_nullifiers(self)?;

        // TODO: these can all be parallel tasks
        for action in self.actions() {
            action.check_stateless(context.clone()).await?;
        }

        // TODO: move these out of component code?
        consensus_rules::stateless::num_clues_equal_to_num_outputs(self)?;
        consensus_rules::stateless::check_memo_exists_if_outputs_absent_if_not(self)?;

        Ok(())
    }

    async fn check_stateful(&self, state: Arc<State>) -> Result<()> {
        claimed_anchor_is_valid(state.clone(), self).await?;
        fmd_parameters_valid(state.clone(), self).await?;

        // TODO: these can all be parallel tasks
        for action in self.actions() {
            action.check_stateful(state.clone()).await?;
        }

        Ok(())
    }

    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        // While we have access to the full Transaction, hash it to
        // obtain a NoteSource we can cache for various actions.
        let source = NoteSource::Transaction { id: self.id() };
        state.object_put("source", source);

        for action in self.actions() {
            action.execute(state).await?;
        }

        Ok(())
    }
}
