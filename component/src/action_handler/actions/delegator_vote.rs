use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{
    action::{DelegatorVote, DelegatorVoteBody},
    Transaction,
};

use crate::{
    governance::{StateReadExt as _, StateWriteExt as _},
    ActionHandler,
};

#[async_trait]
impl ActionHandler for DelegatorVote {
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        let DelegatorVote {
            auth_sig,
            proof,
            body:
                DelegatorVoteBody {
                    start_position,
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

        // 2. Verify the proof against the provided anchor and start position:
        proof
            .verify(anchor, *start_position, *value, *nullifier, *rk)
            .context("a delegator vote proof did not verify")?;

        Ok(())
    }

    async fn check_stateful<S: StateRead>(&self, state: Arc<S>) -> Result<()> {
        let DelegatorVote {
            body:
                DelegatorVoteBody {
                    proposal,
                    vote: _, // All votes are valid, so we don't need to do anything with this
                    start_position,
                    value,
                    unbonded_amount,
                    nullifier,
                    rk: _, // We already used this to check the auth sig in stateless verification
                },
            auth_sig: _, // We already checked this in stateless verification
            proof: _,    // We already checked this in stateless verification
        } = self;

        state.check_proposal_voteable(*proposal).await?;
        state
            .check_proposal_started_at_position(*proposal, *start_position)
            .await?;
        state
            .check_nullifier_unspent_before_start_block_height(*proposal, nullifier)
            .await?;
        state
            .check_nullifier_unvoted_for_proposal(*proposal, nullifier)
            .await?;
        state
            .check_unbonded_amount_correct_exchange_for_proposal(*proposal, value, unbonded_amount)
            .await?;

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let DelegatorVote {
            body:
                DelegatorVoteBody {
                    proposal,
                    vote,
                    nullifier,
                    unbonded_amount,
                    start_position: _, // Not needed to execute: used to check validity of vote
                    value: _,          // Not needed to execute: used to check vote proof
                    rk: _,             // Not needed to execute: used to check auth sig
                },
            ..
        } = self;

        state
            .mark_nullifier_voted_on_proposal(*proposal, nullifier)
            .await?;
        state
            .cast_delegator_vote(*proposal, *vote, nullifier, *unbonded_amount)
            .await?;

        Ok(())
    }
}
