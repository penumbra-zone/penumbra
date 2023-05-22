use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_component::ActionHandler;
use penumbra_crypto::TransactionContext;
use penumbra_proof_params::SPEND_PROOF_VERIFICATION_KEY;
use penumbra_storage::{StateRead, StateWrite};

use crate::{
    component::{NoteManager, StateReadExt},
    Spend,
};

#[async_trait]
impl ActionHandler for Spend {
    type CheckStatelessContext = TransactionContext;
    async fn check_stateless(&self, context: TransactionContext) -> Result<()> {
        let spend = self;
        // 2. Check spend auth signature using provided spend auth key.
        spend
            .body
            .rk
            .verify(context.effect_hash.as_ref(), &spend.auth_sig)
            .context("spend auth signature failed to verify")?;

        // 3. Check that the proof verifies.
        spend
            .proof
            .verify(
                &SPEND_PROOF_VERIFICATION_KEY,
                context.anchor,
                spend.body.balance_commitment,
                spend.body.nullifier,
                spend.body.rk,
            )
            .context("a spend proof did not verify")?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        // Check that the `Nullifier` has not been spent before.
        let spent_nullifier = self.body.nullifier;
        state.check_nullifier_unspent(spent_nullifier).await
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let source = state.object_get("source").unwrap_or_default();

        state.spend_nullifier(self.body.nullifier, source).await;

        Ok(())
    }
}
