use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::sync::StatePayload;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::Output, Transaction};
use tracing::instrument;

use crate::{action_handler::ActionHandler, shielded_pool::NoteManager};

#[async_trait]
impl ActionHandler for Output {
    #[instrument(name = "output", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        let output = self;

        output.proof.verify(
            output.body.balance_commitment,
            output.body.note_payload.note_commitment,
        )?;

        Ok(())
    }

    #[instrument(name = "output", skip(self, _state))]
    async fn check_stateful<S: StateRead>(&self, _state: Arc<S>) -> Result<()> {
        Ok(())
    }

    #[instrument(name = "output", skip(self, state))]
    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let source = state.object_get("source").unwrap_or_default();

        state
            .add_state_payload(StatePayload::Note {
                source,
                note: self.body.note_payload.clone(),
            })
            .await;

        Ok(())
    }
}
