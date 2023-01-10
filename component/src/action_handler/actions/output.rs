use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::sync::StatePayload;
use penumbra_storage::{State, StateRead, StateTransaction};
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
    async fn check_stateful(&self, _state: Arc<State>) -> Result<()> {
        Ok(())
    }

    #[instrument(name = "output", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        let source = state.object_get("source").cloned().unwrap_or_default();

        state
            .add_state_payload(StatePayload::Note {
                source,
                note: self.body.note_payload.clone(),
            })
            .await;

        Ok(())
    }
}
