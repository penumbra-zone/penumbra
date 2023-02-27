use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_chain::{genesis, StateReadExt};
use penumbra_storage::StateWrite;
use penumbra_transaction::action::ProposalPayload;
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
        // TODO: compute intermediate tallies at epoch boundaries (with threshold delegator voting)
        enact_all_passed_proposals(&mut state)
            .await
            .expect("failed to enact proposals");
    }
}

#[instrument(skip(state))]
async fn enact_proposal<S: StateWrite>(mut state: S, payload: &ProposalPayload) -> Result<()> {
    match payload {
        ProposalPayload::Signaling { .. } => {
            // Nothing to do for signaling proposals
        }
        ProposalPayload::Emergency { halt_chain } => {
            // If the proposal calls to halt the chain...
            if *halt_chain {
                // TODO: implement emergency halt
                // // Check to see if the operator has set the environment variable indicating they
                // // wish to resume from this particular chain halt, i.e. the chain has already halted
                // // and they are bringing it back up again
                // if std::env::var("PD_RESUME_FROM_EMERGENCY_HALT_PROPOSAL")
                //     .ok()
                //     .and_then(|v| v.parse::<u64>().ok()) // value of var must be number
                //     .filter(|&resume_from| resume_from == proposal_id) // number must be this proposal's id (to prevent an always-on resume functionality)
                //     .is_some()
                // {
                //     // If so, just print an information message, and don't halt the chain
                //     tracing::info!(proposal = %proposal_id, %height, "resuming from emergency chain halt");
                // } else {
                //     // If not, print an informational message and immediately exit the process
                //     tracing::error!(proposal = %proposal_id, %height, "emergency proposal passed, calling for immediate chain halt");
                //     std::process::exit(0);
                // }
            }
        }
        ProposalPayload::ParameterChange {
            effective_height: _,
            new_parameters,
        } => {
            // TODO: implement immediate parameter change
            // let height = state
            //     .get_block_height()
            //     .await
            //     .context("can get block height")?;

            // // Since other proposals may have changed the chain parameters in the meantime,
            // // and parameter validation must ensure consistency across all parameters, we
            // // need to perform a final validation step prior to applying the new parameters.
            // let old_parameters = state
            //     .get_chain_params()
            //     .await
            //     .context("can get chain parameters")?;

            // if !chain_params::is_valid_stateless(&new_parameters)
            //     || !chain_params::is_valid_stateful(&new_parameters, &old_parameters)
            // {
            //     // The parameters are invalid, so we cannot apply them.
            //     tracing::info!(proposal = %proposal_id, %height, "chain param proposal passed, however the new parameters are invalid");
            //     // TODO: should there be a more descriptive error message here?
            //     return Err(anyhow::anyhow!("invalid chain parameters, could not apply"));
            // }

            // // Apply the new (valid) parameter changes immediately:
            // let new_params = chain_params::resolve_parameters(&new_parameters, &old_parameters)
            //     .context("can resolve validated parameters")?;

            // state.put_chain_params(new_params);
        }
        ProposalPayload::DaoSpend {
            schedule_transactions: _,
            cancel_transactions: _,
        } => {
            // TODO: schedule transaction cancellations by removing the first matching one from the
            // front of the schedule for their effective block
            // TODO: schedule new transactions by appending them to the end of the schedule for their
            // effective block
            // TODO: don't forget to fill in the part in the shielded pool where the transactions
            // actually get included in a block
            todo!("implement daospend execution")
        }
    }

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

                    // If the proposal passes, enact it now
                    if outcome.is_pass() {
                        // IMPORTANT: We **ONLY** enact proposals that have concluded, and whose
                        // tally is `Pass`, and whose state is not `Withdrawn`. This is the sole
                        // place in the codebase where we prevent withdrawn proposals from being
                        // passed!
                        let payload = state
                            .proposal_payload(proposal_id)
                            .await?
                            .context("proposal has payload")?;
                        enact_proposal(&mut state, &payload).await?;
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
