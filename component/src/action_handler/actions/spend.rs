use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{action::Spend, Transaction};
use tracing::instrument;

use crate::{action_handler::ActionHandler, shielded_pool::StateReadExt as _};

#[async_trait]
impl ActionHandler for Spend {
    #[instrument(name = "spend", skip(self, context))]
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        let spend = self;
        let auth_hash = context.transaction_body().auth_hash();
        let anchor = context.anchor;

        // 2. Check spend auth signature using provided spend auth key.
        spend
            .body
            .rk
            .verify(auth_hash.as_ref(), &spend.auth_sig)
            .context("spend auth signature failed to verify")?;

        // 3. Check that the proof verifies.
        spend
            .proof
            .verify(
                anchor,
                spend.body.balance_commitment,
                spend.body.nullifier,
                spend.body.rk,
            )
            .context("a spend proof did not verify")?;

        Ok(())
    }

    #[instrument(name = "spend", skip(self, state))]
    async fn check_stateful(&self, state: Arc<State>) -> Result<()> {
        // Check that the `Nullifier` has not been spent before.
        let spent_nullifier = self.body.nullifier;
        state.check_nullifier_unspent(spent_nullifier).await
    }

    #[instrument(name = "spend", skip(self, _state))]
    async fn execute(&self, _state: &mut StateTransaction) -> Result<()> {
        // Handled in [`crate::action_handler::transaction::Transaction`].

        Ok(())
    }
}
