use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::ActionHandler;
use penumbra_proof_params::SPEND_PROOF_VERIFICATION_KEY;
use penumbra_proto::StateWriteProto as _;
use penumbra_sct::component::{SctManager, SourceContext, StateReadExt as _};
use penumbra_txhash::TransactionContext;

use crate::{event, Spend, SpendProofPublic};

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
        let public = SpendProofPublic {
            anchor: context.anchor,
            balance_commitment: spend.body.balance_commitment,
            nullifier: spend.body.nullifier,
            rk: spend.body.rk,
        };
        spend
            .proof
            .verify(&SPEND_PROOF_VERIFICATION_KEY, public)
            .context("a spend proof did not verify")?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        // Check that the `Nullifier` has not been spent before.
        let spent_nullifier = self.body.nullifier;
        state.check_nullifier_unspent(spent_nullifier).await
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let source = state.get_current_source().expect("source should be set");

        state.nullify(self.body.nullifier, source).await;

        // Also record an ABCI event for transaction indexing.
        state.record_proto(event::spend(&self.body.nullifier));

        Ok(())
    }
}
