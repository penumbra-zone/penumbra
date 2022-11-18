use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{action::Output, Transaction};
use tracing::instrument;

use crate::action_handler::ActionHandler;

#[async_trait]
impl ActionHandler for Output {
    #[instrument(name = "output", skip(self, _context))]
    fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        let output = self;
        if output
            .proof
            .verify(
                output.body.balance_commitment,
                output.body.note_payload.note_commitment,
                output.body.note_payload.ephemeral_key,
            )
            .is_err()
        {
            // TODO should the verification error be bubbled up here?
            return Err(anyhow::anyhow!("An output proof did not verify"));
        }

        Ok(())
    }

    #[instrument(name = "output", skip(self, _state, _context))]
    async fn check_stateful(&self, _state: Arc<State>, _context: Arc<Transaction>) -> Result<()> {
        // No `Output`-specific stateful checks to perform; all checks are
        // performed at the `Transaction` level.
        Ok(())
    }

    #[instrument(name = "output", skip(self, _state))]
    async fn execute(&self, _state: &mut StateTransaction) -> Result<()> {
        // Handled at the `Transaction` level.
        Ok(())
    }
}
