use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_proto::DomainType;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{
    action::{ValidatorVote, ValidatorVoteBody},
    Transaction,
};
use tracing::instrument;

use crate::{
    action_handler::ActionHandler,
    governance::{StateReadExt, StateWriteExt},
};

#[async_trait]
impl ActionHandler for ValidatorVote {
    #[instrument(name = "validator_vote", skip(self, _context))]
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

    #[instrument(name = "validator_vote", skip(self, state))]
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
            .check_validator_has_not_voted(*proposal, identity_key)
            .await?;
        state
            .check_governance_key_matches_validator(identity_key, governance_key)
            .await?;

        Ok(())
    }

    #[instrument(name = "validator_vote", skip(self, state))]
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

        state
            .cast_validator_vote(*proposal, *identity_key, *vote)
            .await;

        tracing::debug!(proposal = %proposal, "cast validator vote");

        Ok(())
    }
}
