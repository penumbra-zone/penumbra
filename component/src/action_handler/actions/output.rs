use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::AnnotatedNotePayload;
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
            output.body.note_payload.ephemeral_key,
        )?;

        Ok(())
    }

    #[instrument(name = "output", skip(self, _state))]
    async fn check_stateful(&self, _state: Arc<State>) -> Result<()> {
        Ok(())
    }

    #[instrument(name = "output", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        let source = state
            .object_get("source")
            .cloned()
            .expect("set in Transaction::execute");
        state
            .add_note(AnnotatedNotePayload {
                source,
                payload: self.body.note_payload.clone(),
            })
            .await;

        Ok(())
    }
}
