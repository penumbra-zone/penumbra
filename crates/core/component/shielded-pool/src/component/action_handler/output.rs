use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::ActionHandler;
use penumbra_proof_params::OUTPUT_PROOF_VERIFICATION_KEY;
use penumbra_proto::StateWriteProto as _;
use penumbra_sct::component::SourceContext;

use crate::{component::NoteManager, event, output::OutputProofPublic, Output};

#[async_trait]
impl ActionHandler for Output {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        let output = self;

        output.proof.verify(
            &OUTPUT_PROOF_VERIFICATION_KEY,
            OutputProofPublic {
                balance_commitment: output.body.balance_commitment,
                note_commitment: output.body.note_payload.note_commitment,
            },
        )?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let source = state
            .get_current_source()
            .expect("source should be set during execution");

        state
            .add_note_payload(self.body.note_payload.clone(), source)
            .await;

        state.record_proto(event::output(&self.body.note_payload));

        Ok(())
    }
}
