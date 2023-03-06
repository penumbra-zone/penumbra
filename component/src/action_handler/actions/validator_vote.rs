use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_chain::StateReadExt as _;
use penumbra_proto::DomainType;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{
    action::{ValidatorVote, ValidatorVoteBody},
    proposal, Transaction,
};

use crate::{
    action_handler::ActionHandler,
    governance::{StateReadExt, StateWriteExt},
};

#[async_trait]
impl ActionHandler for ValidatorVote {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        let ValidatorVote { body, auth_sig } = self;

        // Check the signature using the GOVERNANCE KEY:
        let body_bytes = body.encode_to_vec();
        body.governance_key
            .0
            .verify(&body_bytes, auth_sig)
            .context("validator vote signature failed to verify")?;

        // This is stateless verification, so we still need to check that the proposal being voted
        // on exists, and that this validator hasn't voted on it already.

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        let ValidatorVote {
            body:
                ValidatorVoteBody {
                    proposal,
                    vote: _, // All votes are valid, so we don't need to do anything with this
                    identity_key,
                    governance_key,
                },
            auth_sig: _, // We already checked this in stateless verification
        } = self;

        state.check_proposal_votable(*proposal).await?;
        state
            .check_validator_active_at_proposal_start(*proposal, identity_key)
            .await?;
        state
            .check_validator_has_not_voted(*proposal, identity_key)
            .await?;
        state
            .check_governance_key_matches_validator(identity_key, governance_key)
            .await?;

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let ValidatorVote {
            auth_sig: _,
            body:
                ValidatorVoteBody {
                    proposal,
                    vote,
                    identity_key,
                    governance_key: _, // This is only used for checks so that stateless verification can be done on the signature
                },
        } = self;

        tracing::debug!(proposal = %proposal, "cast validator vote");
        state.cast_validator_vote(*proposal, *identity_key, *vote);

        // If a proposal is an emergency proposal, every validator vote triggers a check to see if
        // we should immediately enact the proposal (if it's reached a 2/3 majority).
        let proposal_state = state
            .proposal_state(*proposal)
            .await?
            .expect("proposal missing state");
        let proposal_payload = state
            .proposal_payload(*proposal)
            .await?
            .expect("proposal missing payload");
        // IMPORTANT: We don't want to enact an emergency proposal if it's been withdrawn, because
        // withdrawal should prevent any proposal, even an emergency proposal, from being enacted.
        if !proposal_state.is_withdrawn() && proposal_payload.is_emergency() {
            tracing::debug!(proposal = %proposal, "proposal is emergency, checking for emergency pass condition");
            let tally = state.current_tally(*proposal).await?;
            let total_voting_power = state
                .total_voting_power_at_proposal_start(*proposal)
                .await?;
            let chain_params = state.get_chain_params().await?;
            if tally.emergency_pass(total_voting_power, &chain_params) {
                // If the emergency pass condition is met, enact the proposal
                tracing::debug!(proposal = %proposal, "emergency pass condition met, trying to enact proposal");
                // Try to enact the proposal based on its payload
                match state.enact_proposal(*proposal, &proposal_payload).await? {
                    Ok(_) => tracing::debug!(proposal = %proposal, "emergency proposal enacted"),
                    Err(error) => {
                        tracing::warn!(proposal = %proposal, %error, "error enacting emergency proposal")
                    }
                }
                // Update the proposal state to reflect the outcome (it will always be passed,
                // because we got to this point)
                state.put_proposal_state(
                    *proposal,
                    proposal::State::Finished {
                        outcome: proposal::Outcome::Passed,
                    },
                );
            }
        }

        Ok(())
    }
}
