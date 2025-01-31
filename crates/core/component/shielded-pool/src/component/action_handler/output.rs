use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sdk_proof_params::OUTPUT_PROOF_VERIFICATION_KEY;
use penumbra_sdk_proto::{DomainType as _, StateWriteProto as _};
use penumbra_sdk_sct::component::source::SourceContext;

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

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let source = state
            .get_current_source()
            .expect("source should be set during execution");

        state
            .add_note_payload(self.body.note_payload.clone(), source.into())
            .await;

        state.record_proto(
            event::EventOutput {
                note_commitment: self.body.note_payload.note_commitment,
            }
            .to_proto(),
        );

        Ok(())
    }
}
