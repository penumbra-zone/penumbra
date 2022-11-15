use std::{collections::BTreeSet, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_crypto::Nullifier;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{action::Spend, Transaction};
use tracing::instrument;

use crate::action_handler::ActionHandler;

#[async_trait]
impl ActionHandler for Spend {
    #[instrument(name = "spend", skip(self, context))]
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        let spend = self;
        let auth_hash = context.transaction_body().auth_hash();
        let anchor = context.anchor;

        // 2. Check all spend auth signatures using provided spend auth keys
        // and check all proofs verify. If any action does not verify, the entire
        // transaction has failed.
        // TODO: store/fetch this in ephemeral
        let mut spent_nullifiers = BTreeSet::<Nullifier>::new();

        spend
            .body
            .rk
            .verify(auth_hash.as_ref(), &spend.auth_sig)
            .context("spend auth signature failed to verify")?;

        spend
            .proof
            .verify(
                anchor,
                spend.body.balance_commitment,
                spend.body.nullifier,
                spend.body.rk,
            )
            .context("a spend proof did not verify")?;

        // Check nullifier has not been revealed already in this transaction.
        if spent_nullifiers.contains(&spend.body.nullifier.clone()) {
            return Err(anyhow::anyhow!("Double spend"));
        }

        spent_nullifiers.insert(spend.body.nullifier);
        // TODO: spent_nullifiers needs to track across actions, place in ephemeral?

        Ok(())
    }

    #[instrument(name = "spend", skip(self, state))]
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        todo!()
    }

    #[instrument(name = "spend", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        todo!()
    }
}
