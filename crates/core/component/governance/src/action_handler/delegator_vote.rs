use std::sync::Arc;

use anyhow::{Context, Result};
use ark_ff::Zero;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use decaf377::Fr;
use penumbra_proof_params::DELEGATOR_VOTE_PROOF_VERIFICATION_KEY;
use penumbra_txhash::TransactionContext;

use crate::{
    DelegatorVote, DelegatorVoteBody, DelegatorVoteProofPublic,
    {component::StateWriteExt, StateReadExt},
};
use cnidarium_component::ActionHandler;

#[async_trait]
impl ActionHandler for DelegatorVote {
    type CheckStatelessContext = TransactionContext;

    async fn check_stateless(&self, context: TransactionContext) -> Result<()> {
        let DelegatorVote {
            auth_sig,
            proof,
            body:
                DelegatorVoteBody {
                    start_position,
                    nullifier,
                    rk,
                    value,
                    // Unused in stateless checks:
                    unbonded_amount: _,
                    vote: _,     // Only used when executing the vote
                    proposal: _, // Checked against the current open proposals statefully
                },
        } = self;

        // 1. Check spend auth signature using provided spend auth key.
        rk.verify(context.effect_hash.as_ref(), auth_sig)
            .context("delegator vote auth signature failed to verify")?;

        // 2. Verify the proof against the provided anchor and start position:
        let public = DelegatorVoteProofPublic {
            anchor: context.anchor,
            balance_commitment: value.commit(Fr::zero()),
            nullifier: *nullifier,
            rk: *rk,
            start_position: *start_position,
        };
        proof
            .verify(&DELEGATOR_VOTE_PROOF_VERIFICATION_KEY, public)
            .context("a delegator vote proof did not verify")?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
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

        state.check_proposal_votable(*proposal).await?;
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
                    value,
                    start_position: _, // Not needed to execute: used to check validity of vote
                    rk: _,             // Not needed to execute: used to check auth sig
                },
            ..
        } = self;

        state
            .mark_nullifier_voted_on_proposal(*proposal, nullifier)
            .await;
        let identity_key = state.validator_by_delegation_asset(value.asset_id).await?;
        state
            .cast_delegator_vote(*proposal, identity_key, *vote, nullifier, *unbonded_amount)
            .await?;

        Ok(())
    }
}
