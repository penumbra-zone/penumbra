use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_chain::{genesis, Epoch, StateReadExt};
use penumbra_storage::StateWrite;
use tendermint::abci;
use tracing::instrument;

use super::{StateReadExt as _, StateWriteExt as _};
use crate::{governance::proposal, shielded_pool::StateWriteExt, Component};

pub struct Governance {}

#[async_trait]
impl Component for Governance {
    #[instrument(name = "governance", skip(state, _app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, _app_state: &genesis::AppState) {
        // Initialize the proposal counter to zero
        state.init_proposal_counter().await;
    }

    #[instrument(name = "governance", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite>(_state: S, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "governance", skip(state, _end_block))]
    async fn end_block<S: StateWrite>(mut state: S, _end_block: &abci::request::EndBlock) {
        // TODO: This will need to be altered to support dynamic epochs
        let height = state.get_block_height().await.unwrap();
        let epoch_duration = state.get_epoch_duration().await.unwrap();
        let epoch = Epoch::from_height(height, epoch_duration);
        if epoch.is_epoch_end(height) {
            end_epoch(&mut state)
                .await
                .expect("end epoch should never fail");
        }
    }
}

async fn end_epoch<S: StateWrite>(mut state: S) -> Result<()> {
    // Every epoch, sweep all delegator votes into tallies (this will be homomorphic in the future)
    state.tally_delegator_votes().await?;
    // Then, enact any proposals that have passed, after considering the tallies to determine what
    // proposals have passed
    enact_all_passed_proposals(&mut state).await?;
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
        if state.height().await
            >= state
                .proposal_voting_end(proposal_id)
                .await?
                .context("proposal has voting end")?
        {
            let current_state = state
                .proposal_state(proposal_id)
                .await?
                .context("proposal has id")?;

            let outcome = match current_state {
                proposal::State::Voting => {
                    // If the proposal is still in the voting state, tally and conclude it (this will
                    // automatically remove it from the list of unfinished proposals)
                    let outcome = state.current_tally(proposal_id).await?.outcome(
                        state.total_voting_power().await?,
                        &state.get_chain_params().await?,
                    );

                    // If the proposal passes, enact it now (or try to: if the proposal can't be
                    // enacted, continue onto the next one without throwing an error, just trace the
                    // error, since proposals are allowed to fail to be enacted)
                    if outcome.is_pass() {
                        // IMPORTANT: We **ONLY** enact proposals that have concluded, and whose
                        // tally is `Pass`, and whose state is not `Withdrawn`. This is the sole
                        // place in the codebase where we prevent withdrawn proposals from being
                        // passed!
                        let payload = state
                            .proposal_payload(proposal_id)
                            .await?
                            .context("proposal has payload")?;
                        match state.enact_proposal(&payload).await? {
                            Ok(()) => {}
                            Err(error) => {
                                tracing::error!(proposal = %proposal_id, %error, "failed to enact proposal");
                            }
                        };
                    }

                    outcome.into()
                }
                proposal::State::Withdrawn { reason } => proposal::Outcome::Failed {
                    withdrawn: proposal::Withdrawn::WithReason { reason },
                },
                proposal::State::Finished { outcome: _ } => {
                    panic!("proposal {proposal_id} is already finished, and should have been removed from the active set");
                }
                proposal::State::Claimed { outcome: _ } => {
                    panic!("proposal {proposal_id} is already claimed, and should have been removed from the active set");
                }
            };

            tracing::info!(proposal = %proposal_id, outcome = ?outcome, "proposal voting concluded");

            // Update the proposal state to reflect the outcome
            state.put_proposal_state(proposal_id, proposal::State::Finished { outcome });
        }
    }

    Ok(())
}
