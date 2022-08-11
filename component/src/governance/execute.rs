use super::proposal;
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
    // TODO: store proposal, refund address, key
    // TODO: schedule start and end of proposal based on current chain parameters for governance
}

pub async fn proposal_withdraw(state: &State, proposal_withdraw: &ProposalWithdraw) {
    // TODO: mark the proposal as withdrawn
    // TODO: handle distinction of pre-voting vs. post-voting withdrawal?
}

pub async fn validator_vote(state: &State, validator_vote: &ValidatorVote) {
    // TODO: record the vote
    // TODO: update the table of validators who voted on this proposal
}

// TODO: fill in when delegator votes happen
// pub async fn delegator_vote(state: &State, delegator_vote: &DelegatorVote) {}
