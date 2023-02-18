use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{
    action::{DelegatorVote, DelegatorVoteBody},
    Transaction,
};

use crate::ActionHandler;

#[async_trait]
impl ActionHandler for DelegatorVote {
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        let DelegatorVote {
            auth_sig,
            proof,
            body:
                DelegatorVoteBody {
                    start_height,
                    value,
                    nullifier,
                    rk,
                    // Unused in stateless checks:
                    vote: _,            // Only used when executing the vote
                    proposal: _,        // Checked against the current open proposals statefully
                    unbonded_amount: _, // Checked against the proposal's snapshot exchange rate statefully
                },
        } = self;

        let effect_hash = context.transaction_body().effect_hash();
        let anchor = context.anchor;

        // 1. Check spend auth signature using provided spend auth key.
        rk.verify(effect_hash.as_ref(), auth_sig)
            .context("delegator vote auth signature failed to verify")?;

        // 2. Check that the proof verifies.
        proof
            .verify(anchor, *start_height, *value, *nullifier, *rk)
            .context("a delegator vote proof did not verify")?;

        Ok(())
    }

    async fn check_stateful<S: StateRead>(&self, _state: Arc<S>) -> Result<()> {
        // TODO:
        // 1. Check that the proposal exists and is open for voting.
        // 2. Check that the `Nullifier` has not been spent before for this proposal.
        // 3. Check that the `value` is a delegation token and converts to the `unbonded_amount`
        //    using the exchange rate for the validator of `value` as of the moment when the
        //    proposal started.
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, _state: S) -> Result<()> {
        Ok(())
    }
}
