use std::sync::Arc;

use crate::{event, genesis};
use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sdk_proto::StateWriteProto as _;
use tendermint::v0_37::abci;
use tracing::instrument;

use cnidarium_component::Component;

use crate::{
    proposal_state::{
        Outcome as ProposalOutcome, State as ProposalState, Withdrawn as ProposalWithdrawn,
    },
    tally,
};

mod view;

pub mod rpc;

pub use view::StateReadExt;
pub use view::StateWriteExt;

use penumbra_sdk_sct::component::clock::EpochRead;

pub struct Governance {}

#[async_trait]
impl Component for Governance {
    type AppState = genesis::Content;

    #[instrument(name = "governance", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            Some(genesis) => {
                state.put_governance_params(genesis.governance_params.clone());
                // Clients need to be able to read the next proposal number, even when no proposals have
                // been submitted yet
                state.init_proposal_counter();
            }
            None => {}
        }
    }

    #[instrument(name = "governance", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "governance", skip(state, _end_block))]
    async fn end_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
        let mut state = Arc::get_mut(state).expect("state should be unique");
        // Then, enact any proposals that have passed, after considering the tallies to determine what
        // proposals have passed. Note that this occurs regardless of whether it's the end of an
        // epoch, because proposals can finish at any time.
        enact_all_passed_proposals(&mut state)
            .await
            .expect("enacting proposals should never fail");
    }

    #[instrument(name = "governance", skip(state))]
    async fn end_epoch<S: StateWrite + 'static>(state: &mut Arc<S>) -> Result<()> {
        let state = Arc::get_mut(state).expect("state should be unique");
        state.tally_delegator_votes(None).await?;
        Ok(())
    }
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
            ProposalState::Voting => {
                // If the proposal is still in the voting state, tally and conclude it (this will
                // automatically remove it from the list of unfinished proposals)
                let outcome = state.current_tally(proposal_id).await?.outcome(
                    state
                        .total_voting_power_at_proposal_start(proposal_id)
                        .await?,
                    &state.get_governance_params().await?,
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

                        let proposal =
                            state
                                .proposal_definition(proposal_id)
                                .await?
                                .ok_or_else(|| {
                                    anyhow::anyhow!("proposal {} does not exist", proposal_id)
                                })?;
                        state.record_proto(event::proposal_passed(&proposal));
                    }
                    tally::Outcome::Fail => {
                        tracing::info!(proposal = %proposal_id, "proposal failed");

                        let proposal =
                            state
                                .proposal_definition(proposal_id)
                                .await?
                                .ok_or_else(|| {
                                    anyhow::anyhow!("proposal {} does not exist", proposal_id)
                                })?;
                        state.record_proto(event::proposal_failed(&proposal));
                    }
                    tally::Outcome::Slash => {
                        tracing::info!(proposal = %proposal_id, "proposal slashed");

                        let proposal =
                            state
                                .proposal_definition(proposal_id)
                                .await?
                                .ok_or_else(|| {
                                    anyhow::anyhow!("proposal {} does not exist", proposal_id)
                                })?;
                        state.record_proto(event::proposal_slashed(&proposal));
                    }
                }

                outcome.into()
            }
            ProposalState::Withdrawn { reason } => {
                tracing::info!(proposal = %proposal_id, reason = ?reason, "proposal concluded after being withdrawn");
                ProposalOutcome::Failed {
                    withdrawn: ProposalWithdrawn::WithReason { reason },
                }
            }
            ProposalState::Finished { outcome: _ } => {
                anyhow::bail!("proposal {proposal_id} is already finished, and should have been removed from the active set");
            }
            ProposalState::Claimed { outcome: _ } => {
                anyhow::bail!("proposal {proposal_id} is already claimed, and should have been removed from the active set");
            }
        };

        // Update the proposal state to reflect the outcome
        state.put_proposal_state(proposal_id, ProposalState::Finished { outcome });
    }

    Ok(())
}
