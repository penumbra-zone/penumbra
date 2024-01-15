use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::ActionHandler;
use penumbra_sct::CommitmentSource;
use penumbra_shielded_pool::component::NoteManager;

use crate::CommunityPoolOutput;

#[async_trait]
impl ActionHandler for CommunityPoolOutput {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Any output from the Community Pool is valid (it's just a transparent output).
        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // Any output from the Community Pool is valid (it's just a transparent output).
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // Executing a Community Pool output is just minting a note to the recipient of the output.
        state
            .mint_note(
                self.value,
                &self.address,
                CommitmentSource::CommunityPoolOutput,
            )
            .await
    }
}
