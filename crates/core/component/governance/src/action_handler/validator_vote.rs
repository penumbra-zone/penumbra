use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_proto::DomainType;

use crate::component::StateWriteExt;
use crate::{action_handler::ActionHandler, StateReadExt};
use crate::{
    proposal_state::Outcome,
    proposal_state::State as ProposalState,
    {ValidatorVote, ValidatorVoteBody, MAX_VALIDATOR_VOTE_REASON_LENGTH},
};

#[async_trait]
impl ActionHandler for ValidatorVote {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        let ValidatorVote { body, auth_sig } = self;

        // Check the signature using the GOVERNANCE KEY:
        let body_bytes = body.encode_to_vec();
        body.governance_key
            .0
            .verify(&body_bytes, auth_sig)
            .context("validator vote signature failed to verify")?;

        // Check the length of the validator reason field.
        if body.reason.0.len() > MAX_VALIDATOR_VOTE_REASON_LENGTH {
            anyhow::bail!("validator vote reason is too long");
        }

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
                    reason: _, // Checked the length in the stateless verification
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
                    reason,
                },
        } = self;

        let proposal_state = state
            .proposal_state(*proposal)
            .await?
            .expect("proposal missing state");

        // TODO(erwan): Keeping this guard here, because there was previously a comment highlighting
        // that we especially do _not_ want to ennact proposals that have been withdrawn. However, note
        // that `stateful` verification checks that the proposal is votable and we're executing against
        // the same state, so this seem redundant.
        // I will remove it once in the PR review once this is confirmed.
        if proposal_state.is_withdrawn() {
            tracing::debug!(validator_identity = %identity_key, proposal = %proposal, "cannot cast a vote for a withdrawn proposal");
            return Ok(());
        }

        tracing::debug!(validator_identity = %identity_key, proposal = %proposal, "cast validator vote");
        state.cast_validator_vote(*proposal, *identity_key, *vote, reason.clone());

        // Certain proposals are considered "emergency" proposals, and are enacted immediately if they
        // receive +2/3 of the votes. These proposals are: `IbcFreeze`, `IbcUnfreeze`, and `Emergency`.
        let proposal_payload = state
            .proposal_payload(*proposal)
            .await?
            .expect("proposal missing payload");

        if proposal_payload.is_emergency() || proposal_payload.is_ibc_freeze() {
            tracing::debug!(proposal = %proposal, "proposal is emergency-tier, checking for emergency pass condition");
            let tally = state.current_tally(*proposal).await?;
            let total_voting_power = state
                .total_voting_power_at_proposal_start(*proposal)
                .await?;
            let governance_params = state.get_governance_params().await?;
            if tally.emergency_pass(total_voting_power, &governance_params) {
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
                    ProposalState::Finished {
                        outcome: Outcome::Passed,
                    },
                );
            }
        }

        Ok(())
    }
}
