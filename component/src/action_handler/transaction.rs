use std::{collections::BTreeSet, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_chain::{AnnotatedNotePayload, NoteSource, StateReadExt as _};
use penumbra_storage::{State, StateTransaction, StateWrite as _};
use penumbra_transaction::Transaction;

use crate::{
    shielded_pool::{
        consensus_rules, event, NoteManager as _, StateReadExt as _, StateWriteExt as _,
    },
    stake::StateReadExt as _,
};

use super::ActionHandler;

#[async_trait]
impl ActionHandler for Transaction {
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?
        let auth_hash = context.transaction_body().auth_hash();

        // 1. Check binding signature.
        self.binding_verification_key()
            .verify(auth_hash.as_ref(), self.binding_sig())
            .context("binding signature failed to verify")?;

        // 2. Disallow multiple `Spend`s with the same `Nullifier`.
        // This can't be implemented in the (`Spend`)[`crate::action_handler::actions::spend::Spend`] `ActionHandler`
        // because it requires access to the entire transaction, and we don't want to perform the check across the entire
        // transaction for *each* `Spend` within the transaction, only once.
        let mut spent_nullifiers = BTreeSet::new();
        for nf in self.spent_nullifiers() {
            if let Some(duplicate) = spent_nullifiers.replace(nf) {
                return Err(anyhow::anyhow!(
                    "Duplicate nullifier in transaction: {}",
                    duplicate
                ));
            }
        }

        // 3. Check that the transaction undelegates from at most one validator.
        let undelegation_identities = self
            .undelegations()
            .map(|u| u.validator_identity.clone())
            .collect::<BTreeSet<_>>();

        if undelegation_identities.len() > 1 {
            return Err(anyhow::anyhow!(
                "transaction contains undelegations from multiple validators: {:?}",
                undelegation_identities
            ));
        }

        // We prohibit actions other than `Spend`, `Delegate`, `Output` and `Undelegate` in
        // transactions that contain `Undelegate`, to avoid having to quarantine them.
        if undelegation_identities.len() == 1 {
            use penumbra_transaction::Action::*;
            for action in self.transaction_body().actions {
                if !matches!(action, Undelegate(_) | Delegate(_) | Spend(_) | Output(_)) {
                    return Err(anyhow::anyhow!("transaction contains an undelegation, but also contains an action other than Spend, Delegate, Output or Undelegate"));
                }
            }
        }

        // TODO: these can all be parallel tasks
        for action in self.actions() {
            action.check_stateless(context.clone())?;
        }

        consensus_rules::stateless::num_clues_equal_to_num_outputs(self)?;
        consensus_rules::stateless::check_memo_exists_if_outputs_absent_if_not(self)?;

        Ok(())
    }

    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        state.check_claimed_anchor(self.anchor).await?;

        let previous_fmd_parameters = state
            .get_previous_fmd_parameters()
            .await
            .expect("chain params request must succeed");
        let current_fmd_parameters = state
            .get_current_fmd_parameters()
            .await
            .expect("chain params request must succeed");
        let height = state.get_block_height().await?;
        consensus_rules::stateful::fmd_precision_within_grace_period(
            self,
            previous_fmd_parameters,
            current_fmd_parameters,
            height,
        )?;

        // TODO: these can all be parallel tasks
        for action in self.actions() {
            action
                .check_stateful(state.clone(), context.clone())
                .await?;
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
