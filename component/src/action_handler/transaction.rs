use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::NoteSource;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;
use tokio::task::JoinSet;
use tracing::{instrument, Instrument};

use crate::shielded_pool::consensus_rules;

use self::stateful::{claimed_anchor_is_valid, fmd_parameters_valid, timestamps_are_valid};

use super::ActionHandler;

mod stateful;
mod stateless;

#[cfg(test)]
mod tests;

use stateless::{no_duplicate_nullifiers, valid_binding_signature};

#[async_trait]
impl ActionHandler for Transaction {
    // We only instrument the top-level `check_stateless`, so we get one span for each transaction.
    #[instrument(skip(self, context))]
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?

        // TODO: unify code organization
        valid_binding_signature(self)?;
        no_duplicate_nullifiers(self)?;
        consensus_rules::stateless::num_clues_equal_to_num_outputs(self)?;
        consensus_rules::stateless::check_memo_exists_if_outputs_absent_if_not(self)?;

        // Currently, we need to clone the component actions so that the spawned
        // futures can have 'static lifetimes. In the future, we could try to
        // use the yoke crate, but cloning is almost certainly not a big deal
        // for now.
        let mut action_checks = JoinSet::new();
        for (i, action) in self.actions().cloned().enumerate() {
            let context2 = context.clone();
            let span = action.create_span(i);
            action_checks
                .spawn(async move { action.check_stateless(context2).await }.instrument(span));
        }
        // Now check if any component action failed verification.
        while let Some(check) = action_checks.join_next().await {
            check??;
        }

        Ok(())
    }

    // We only instrument the top-level `check_stateful`, so we get one span for each transaction.
    #[instrument(skip(self, state))]
    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        claimed_anchor_is_valid(state.clone(), self).await?;
        fmd_parameters_valid(state.clone(), self).await?;
        timestamps_are_valid(state.clone(), self).await?;

        // Currently, we need to clone the component actions so that the spawned
        // futures can have 'static lifetimes. In the future, we could try to
        // use the yoke crate, but cloning is almost certainly not a big deal
        // for now.
        let mut action_checks = JoinSet::new();
        for (i, action) in self.actions().cloned().enumerate() {
            let state2 = state.clone();
            let span = action.create_span(i);
            action_checks
                .spawn(async move { action.check_stateful(state2).await }.instrument(span));
        }
        // Now check if any component action failed verification.
        while let Some(check) = action_checks.join_next().await {
            check??;
        }

        Ok(())
    }

    // We only instrument the top-level `execute`, so we get one span for each transaction.
    #[instrument(skip(self, state))]
    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // While we have access to the full Transaction, hash it to
        // obtain a NoteSource we can cache for various actions.
        let source = NoteSource::Transaction { id: self.id().0 };
        state.object_put("source", source);

        for (i, action) in self.actions().enumerate() {
            let span = action.create_span(i);
            action.execute(&mut state).instrument(span).await?;
        }

        // Delete the note source, in case someone else tries to read it.
        state.object_delete("source");

        Ok(())
    }
}
