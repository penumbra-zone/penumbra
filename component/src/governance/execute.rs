use super::{proposal, View as _};
use penumbra_chain::View as _;
use penumbra_crypto::rdsa::{SpendAuth, VerificationKey};
use penumbra_storage::State;
use penumbra_transaction::action::{
    ProposalSubmit, ProposalWithdraw, ProposalWithdrawBody, ValidatorVote, ValidatorVoteBody,
};

pub async fn proposal_submit(
    state: &State,
    ProposalSubmit {
        proposal,
        deposit_amount: _, // We don't need to do anything with the deposit amount, it's paid by a Spend
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

    // Set the withdrawal key for the proposal
    state
        .put_withdrawal_key(proposal_id, *withdraw_proposal_key)
        .await;

    // Set the proposal state to voting (votes start immediately)
    state
        .put_proposal_state(proposal_id, proposal::State::Voting)
        .await
        .expect("can set proposal state");

    // Determine what block it is currently, and calculate when the proposal should finish voting,
    // then write that into the state
    let chain_params = state
        .get_chain_params()
        .await
        .expect("can get chain params");
    let current_block = state
        .get_block_height()
        .await
        .expect("can get block height");
    let voting_end = current_block + chain_params.proposal_voting_blocks;
    state.put_proposal_voting_end(proposal_id, voting_end).await;

    tracing::debug!(proposal = %proposal_id, "created proposal");
}

pub async fn proposal_withdraw(state: &State, proposal_withdraw: &ProposalWithdraw) {
    // TODO: mark the proposal as withdrawn
    // TODO: handle distinction of pre-voting vs. post-voting withdrawal?
}

pub async fn validator_vote(state: &State, validator_vote: &ValidatorVote) {
    // TODO: record the vote
    // TODO: update the table of validators who voted on this proposal
}

pub async fn conclude_proposal(state: &State, proposal_id: u64) {
    // TODO: tally delegator votes
    // TODO: tally weighted validator votes
    // TODO: write the outcome of the proposal into the state
}

// TODO: fill in when delegator votes happen
// pub async fn delegator_vote(state: &State, delegator_vote: &DelegatorVote) {}
