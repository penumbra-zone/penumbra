use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::sync::StatePayload;
use penumbra_proof_params::OUTPUT_PROOF_VERIFICATION_KEY;
use penumbra_storage::{StateRead, StateWrite};

use penumbra_transaction::action::Output;

use crate::{action_handler::ActionHandler, shielded_pool::NoteManager};

#[async_trait]
impl ActionHandler for Output {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        let output = self;

        output.proof.verify(
            &OUTPUT_PROOF_VERIFICATION_KEY,
            output.body.balance_commitment,
            output.body.note_payload.note_commitment,
        )?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        Ok(())
    }

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
