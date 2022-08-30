use crate::governance::proposal::Outcome;

use super::{proposal, tally, View as _};
use penumbra_chain::View as _;
use penumbra_storage::State;
use penumbra_transaction::action::{
    ProposalPayload, ProposalSubmit, ProposalWithdraw, ProposalWithdrawBody, ValidatorVote,
    ValidatorVoteBody,
};
use tracing::instrument;

#[instrument(skip(state))]
pub async fn proposal_submit(
    state: &State,
    ProposalSubmit {
        proposal,
        deposit_amount,
        deposit_refund_address,
        withdraw_proposal_key,
    }: &ProposalSubmit,
) {
    // Store the contents of the proposal and generate a fresh proposal id for it
    let proposal_id = state
        .new_proposal(proposal)
        .await
        .expect("can create proposal");

    // Set the refund address for the proposal
    state
        .put_refund_address(proposal_id, *deposit_refund_address)
        .await;

    // Set the deposit amount for the proposal
    state.put_deposit_amount(proposal_id, *deposit_amount).await;

    // Set the withdrawal key for the proposal
    state
        .put_withdrawal_key(proposal_id, *withdraw_proposal_key)
        .await;

    // Set the proposal state to voting (votes start immediately)
    state
        .put_proposal_state(proposal_id, proposal::State::Voting)
        .await
        .expect("can set proposal state");

    // Determine what block it is currently, and calculate when the proposal should start voting
    // (now!) and finish voting (later...), then write that into the state
    let chain_params = state
        .get_chain_params()
        .await
        .expect("can get chain params");
    let current_block = state
        .get_block_height()
        .await
        .expect("can get block height");
    let voting_end = current_block + chain_params.proposal_voting_blocks;
    state
        .put_proposal_voting_start(proposal_id, current_block)
        .await;
    state.put_proposal_voting_end(proposal_id, voting_end).await;

    tracing::debug!(proposal = %proposal_id, "created proposal");
}

#[instrument(skip(state))]
pub async fn proposal_withdraw(
    state: &State,
    ProposalWithdraw {
        auth_sig: _,
        body: ProposalWithdrawBody { proposal, reason },
    }: &ProposalWithdraw,
) {
    state
        .put_proposal_state(
            *proposal,
            proposal::State::Withdrawn {
                reason: reason.clone(),
            },
        )
        .await
        .expect("proposal withdraw succeeds");

    tracing::debug!(proposal = %proposal, "withdrew proposal");
}

#[instrument(skip(state))]
pub async fn validator_vote(
    state: &State,
    ValidatorVote {
        auth_sig: _,
        body:
            ValidatorVoteBody {
                proposal,
                vote,
                identity_key,
                governance_key: _, // This is only used for checks so that stateless verification can be done on the signature
            },
    }: &ValidatorVote,
) {
    state
        .cast_validator_vote(*proposal, *identity_key, *vote)
        .await;

    tracing::debug!(proposal = %proposal, "cast validator vote");
}

// TODO: fill in when delegator votes happen
// pub async fn delegator_vote(state: &State, delegator_vote: &DelegatorVote) {}

#[instrument(skip(state))]
pub async fn enact_all_passed_proposals(state: &State) {
    let parameters = tally::Parameters::new(state)
        .await
        .expect("can generate tally parameters");

    let height = state
        .get_block_height()
        .await
        .expect("can get block height");

    let circumstance = tally::Circumstance::new(state)
        .await
        .expect("can generate tally circumstance");

    // For every unfinished proposal, conclude those that finish in this block
    for proposal_id in state
        .unfinished_proposals()
        .await
        .expect("can get unfinished proposals")
    {
        // TODO: tally delegator votes
        if let Some(outcome) = parameters
            .tally(state, circumstance, proposal_id)
            .await
            .expect("can tally proposal")
        {
            tracing::debug!(proposal = %proposal_id, outcome = ?outcome, "proposal voting finished");

            // If the outcome was not vetoed, issue a refund of the proposal deposit --
            // otherwise, the deposit will never be refunded, and therefore is burned
            if outcome.should_be_refunded() {
                tracing::debug!(proposal = %proposal_id, "issuing proposal deposit refund");
                state
                    .add_proposal_refund(height, proposal_id)
                    .await
                    .expect("can add proposal refund");
            } else {
                tracing::debug!(proposal = %proposal_id, "burning proposal deposit for vetoed proposal");
            }

            // If the proposal passes, enact it now
            if outcome.is_passed() {
                enact_proposal(state, proposal_id).await;
            }

            // Log the result
            tracing::info!(proposal = %proposal_id, outcome = match outcome {
                Outcome::Passed => "passed",
                Outcome::Failed { .. } => "failed",
                Outcome::Vetoed {.. } => "vetoed",
            }, "voting concluded");

            // Record the outcome of the proposal: this is especially important for emergency
            // proposals, because it prevents the vote from continuing after they are passed
            state
                .put_proposal_state(proposal_id, proposal::State::Finished { outcome })
                .await
                .expect("can put finished proposal outcome");
        }
    }
}

#[instrument(skip(state))]
async fn enact_proposal(state: &State, proposal_id: u64) {
    let payload = state
        .proposal_payload(proposal_id)
        .await
        .expect("can get proposal payload")
        .expect("proposal payload is present");

    match payload {
        ProposalPayload::Signaling { .. } => {
            // Nothing to do for signaling proposals
        }
        ProposalPayload::Emergency { halt_chain } => {
            let height = state
                .get_block_height()
                .await
                .expect("can get block height");

            if halt_chain {
                tracing::error!(proposal = %proposal_id, %height, "emergency proposal passed, calling for immediate chain halt");
                std::process::exit(0);
            }
        }
        ProposalPayload::ParameterChange {
            effective_height: _,
            new_parameters: _,
        } => todo!("implement parameter change execution"),
        ProposalPayload::DaoSpend {
            schedule_transactions: _,
            cancel_transactions: _,
        } => todo!("implement daospend execution"),
    }
}
