use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::{AnnotatedNotePayload, NoteSource};
use penumbra_storage::{State, StateTransaction, StateWrite as _};
use penumbra_transaction::Transaction;

use crate::{
    shielded_pool::{
        consensus_rules, event, NoteManager as _, StateReadExt as _, StateWriteExt as _,
    },
    stake::StateReadExt as _,
};

use self::stateful::{claimed_anchor_is_valid, fmd_parameters_valid};

use super::ActionHandler;

mod stateful;
mod stateless;

use stateless::{at_most_one_undelegate, no_duplicate_nullifiers, valid_binding_signature};

#[async_trait]
impl ActionHandler for Transaction {
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?

        valid_binding_signature(self)?;
        no_duplicate_nullifiers(self)?;
        at_most_one_undelegate(self)?;

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
        let source = NoteSource::Transaction { id: self.id() };

        if let Some((epoch, identity_key)) = state.should_quarantine(self).await {
            for quarantined_output in self.note_payloads().cloned() {
                // Queue up scheduling this note to be unquarantined: the actual state-writing for
                // all quarantined notes happens during end_block, to avoid state churn
                state
                    .schedule_note(epoch, identity_key, quarantined_output, source)
                    .await;
            }
            for quarantined_spent_nullifier in self.spent_nullifiers() {
                state
                    .quarantined_spend_nullifier(
                        epoch,
                        identity_key,
                        quarantined_spent_nullifier,
                        source,
                    )
                    .await;
                state.record(event::quarantine_spend(quarantined_spent_nullifier));
            }
        } else {
            for payload in self.note_payloads().cloned() {
                state
                    .add_note(AnnotatedNotePayload { payload, source })
                    .await;
            }
            for spent_nullifier in self.spent_nullifiers() {
                state.spend_nullifier(spent_nullifier, source).await;
                state.record(event::spend(spent_nullifier));
            }
        }

        // If there was any proposal submitted in the block, ensure we track this so that clients
        // can retain state needed to vote as delegators
        if self.proposal_submits().next().is_some() {
            let mut compact_block = state.stub_compact_block();
            compact_block.proposal_started = true;
            state.stub_put_compact_block(compact_block);
        }

        for action in self.actions() {
            action.execute(state).await?;
        }

        Ok(())
    }
}
