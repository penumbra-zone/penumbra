use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_chain::{genesis, StateReadExt};
use penumbra_storage::StateWrite;
use penumbra_transaction::proposal;
use tendermint::abci;
use tracing::instrument;

use super::{tally, StateReadExt as _, StateWriteExt as _};
use crate::Component;

pub struct Governance {}

#[async_trait]
impl Component for Governance {
    #[instrument(name = "governance", skip(state, _app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, _app_state: &genesis::AppState) {
        // Clients need to be able to read the next proposal number, even when no proposals have
        // been submitted yet
        state.init_proposal_counter();
    }

    #[instrument(name = "governance", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite>(_state: S, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "governance", skip(state, _end_block))]
    async fn end_block<S: StateWrite>(mut state: S, _end_block: &abci::request::EndBlock) {
        // Then, enact any proposals that have passed, after considering the tallies to determine what
        // proposals have passed. Note that this occurs regardless of whether it's the end of an
        // epoch, because proposals can finish at any time.
        enact_all_passed_proposals(&mut state)
            .await
            .expect("enacting proposals should never fail");

        // TODO: This will need to be altered to support dynamic epochs
        if state.epoch().await.unwrap().is_epoch_end(
            state
                .get_block_height()
                .await
                .expect("block height should be set"),
        ) {
            end_epoch(&mut state)
                .await
                .expect("end epoch should never fail");
        }
    }
}

async fn end_epoch<S: StateWrite>(mut state: S) -> Result<()> {
    // Every epoch, sweep all delegator votes into tallies (this will be homomorphic in the future)
    state.tally_delegator_votes(None).await?;
    Ok(())
}

#[instrument(skip(state))]
pub async fn enact_all_passed_proposals<S: StateWrite>(mut state: S) -> Result<()> {
    // For every unfinished proposal, conclude those that finish in this block
    for proposal_id in state
        .unfinished_proposals()
        .await
        .context("can get unfinished proposals")?
    {
        // TODO: this check will need to be altered when proposals have clock-time end times
        let proposal_ready = state
            .get_block_height()
            .await
            .expect("block height must be set")
            >= state
                .proposal_voting_end(proposal_id)
                .await?
                .context("proposal has voting end")?;

        if !proposal_ready {
            continue;
        }

        // Do a final tally of any pending delegator votes for the proposal
        state.tally_delegator_votes(Some(proposal_id)).await?;

        let current_state = state
            .proposal_state(proposal_id)
            .await?
            .context("proposal has id")?;

        let outcome = match current_state {
            proposal::State::Voting => {
                // If the proposal is still in the voting state, tally and conclude it (this will
                // automatically remove it from the list of unfinished proposals)
                let outcome = state.current_tally(proposal_id).await?.outcome(
                    state
                        .total_voting_power_at_proposal_start(proposal_id)
                        .await?,
                    &state.get_chain_params().await?,
                );

                // If the proposal passes, enact it now (or try to: if the proposal can't be
                // enacted, continue onto the next one without throwing an error, just trace the
                // error, since proposals are allowed to fail to be enacted)
                match outcome {
                    tally::Outcome::Pass => {
                        // IMPORTANT: We **ONLY** enact proposals that have concluded, and whose
                        // tally is `Pass`, and whose state is not `Withdrawn`. This is the sole
                        // place in the codebase where we prevent withdrawn proposals from being
                        // passed!
                        let payload = state
                            .proposal_payload(proposal_id)
                            .await?
                            .context("proposal has payload")?;
                        match state.enact_proposal(proposal_id, &payload).await? {
                            Ok(()) => {
                                tracing::info!(proposal = %proposal_id, "proposal passed and enacted successfully");
                            }
                            Err(error) => {
                                tracing::warn!(proposal = %proposal_id, %error, "proposal passed but failed to enact");
                            }
                        };
                    }
                    tally::Outcome::Fail => {
                        tracing::info!(proposal = %proposal_id, "proposal failed");
                    }
                    tally::Outcome::Slash => {
                        tracing::info!(proposal = %proposal_id, "proposal slashed");
                    }
                }

                outcome.into()
            }
            proposal::State::Withdrawn { reason } => {
                tracing::info!(proposal = %proposal_id, reason = ?reason, "proposal concluded after being withdrawn");
                proposal::Outcome::Failed {
                    withdrawn: proposal::Withdrawn::WithReason { reason },
                }
            }
            proposal::State::Finished { outcome: _ } => {
                anyhow::bail!("proposal {proposal_id} is already finished, and should have been removed from the active set");
            }
            proposal::State::Claimed { outcome: _ } => {
                anyhow::bail!("proposal {proposal_id} is already claimed, and should have been removed from the active set");
            }
        };

        // Update the proposal state to reflect the outcome
        state.put_proposal_state(proposal_id, proposal::State::Finished { outcome });
    }

    Ok(())
}
