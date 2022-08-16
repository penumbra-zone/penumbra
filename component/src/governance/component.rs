use std::collections::BTreeSet;

use anyhow::{Context as _, Result};
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_crypto::{
    rdsa::{SpendAuth, VerificationKey},
    Address, IdentityKey,
};
use penumbra_storage::{State, StateExt};
use penumbra_transaction::{
    action::{Proposal, Vote},
    Transaction,
};
use tendermint::abci;
use tracing::instrument;

use crate::{Component, Context};

use super::{check, event, execute, metrics, proposal, state_key};

pub struct Governance {
    state: State,
}

#[async_trait]
impl Component for Governance {
    #[instrument(name = "governance", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {}

    #[instrument(name = "governance", skip(self, _ctx, _begin_block))]
    async fn begin_block(&mut self, _ctx: Context, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "governance", skip(_ctx, tx))]
    fn check_tx_stateless(_ctx: Context, tx: &Transaction) -> Result<()> {
        for proposal_submit in tx.proposal_submits() {
            check::stateless::proposal_submit(proposal_submit)?;
        }
        for proposal_withdraw in tx.proposal_withdraws() {
            check::stateless::proposal_withdraw(proposal_withdraw)?;
        }
        for validator_vote in tx.validator_votes() {
            check::stateless::validator_vote(validator_vote)?;
        }
        // TODO: fill in when delegator votes happen
        // for delegator_vote in tx.delegator_votes() {
        //     check::stateless::delegator_vote(delegator_vote)?;
        // }

        Ok(())
    }

    #[instrument(name = "governance", skip(self, _ctx, tx))]
    async fn check_tx_stateful(&self, _ctx: Context, tx: &Transaction) -> Result<()> {
        for proposal_submit in tx.proposal_submits() {
            check::stateful::proposal_submit(&self.state, proposal_submit).await?;
        }
        for proposal_withdraw in tx.proposal_withdraws() {
            check::stateful::proposal_withdraw(&self.state, proposal_withdraw).await?;
        }
        for validator_vote in tx.validator_votes() {
            check::stateful::validator_vote(&self.state, validator_vote).await?;
        }
        // TODO: fill in when delegator votes happen
        // for delegator_vote in tx.delegator_votes() {
        //     check::stateful::delegator_vote(&self.state, delegator_vote).await?;
        // }

        Ok(())
    }

    #[instrument(name = "governance", skip(self, _ctx, tx))]
    async fn execute_tx(&mut self, _ctx: Context, tx: &Transaction) {
        for proposal_submit in tx.proposal_submits() {
            execute::proposal_submit(&self.state, proposal_submit).await;
        }
        for proposal_withdraw in tx.proposal_withdraws() {
            execute::proposal_withdraw(&self.state, proposal_withdraw).await;
        }
        for validator_vote in tx.validator_votes() {
            execute::validator_vote(&self.state, validator_vote).await;
        }
        // TODO: fill in when delegator votes happen
        // for delegator_vote in tx.delegator_votes() {
        //     execute::delegator_vote(&self.state, delegator_vote).await;
        // }
    }

    #[instrument(name = "governance", skip(self, _ctx, end_block))]
    async fn end_block(&mut self, _ctx: Context, end_block: &abci::request::EndBlock) {
        let height: u64 = end_block.height.try_into().unwrap();

        // For every unfinished proposal, conclude those that finish in this block
        for proposal_id in self
            .state
            .unfinished_proposals()
            .await
            .expect("can get unfinished proposals")
        {
            let proposal_end_block = self
                .state
                .proposal_voting_end(proposal_id)
                .await
                .expect("can read proposal end block")
                .expect("proposal exists");
            if proposal_end_block == height {
                execute::conclude_proposal(&self.state, proposal_id).await;
            }
        }

        // TODO: Compute intermediate tallies at epoch boundaries (with threshold delegator voting)
    }
}

impl Governance {
    pub async fn new(state: State) -> Self {
        Self { state }
    }
}

#[async_trait]
pub trait View: StateExt {
    /// Get the id of the next proposal in the sequence of ids.
    async fn next_proposal_id(&self) -> Result<u64> {
        Ok(self
            .get_proto::<u64>(state_key::latest_proposal_id().into())
            .await?
            .map(|i| i + 1)
            .unwrap_or(0))
    }

    /// Store a new proposal with a new proposal id.
    async fn new_proposal(&self, proposal: &Proposal) -> Result<u64> {
        let proposal_id = self.next_proposal_id().await?;

        // Record this proposal id, so we won't re-use it
        self.put_proto(state_key::latest_proposal_id().into(), proposal_id)
            .await;

        // Store the proposal description
        self.put_proto(
            state_key::proposal_description(proposal_id).into(),
            proposal.description.clone(),
        )
        .await;

        // Store the proposal payload
        self.put_domain(
            state_key::proposal_payload(proposal_id).into(),
            proposal.payload.clone(),
        )
        .await;

        // Return the new proposal id
        Ok(proposal_id)
    }

    /// Store the deposit refund address for a proposal.
    async fn put_refund_address(&self, proposal_id: u64, address: Address) {
        self.put_domain(
            state_key::proposal_deposit_refund_address(proposal_id).into(),
            address,
        )
        .await
    }

    /// Store the proposal withdrawal key for a proposal.
    async fn put_withdrawal_key(&self, proposal_id: u64, key: VerificationKey<SpendAuth>) {
        self.put_domain(state_key::proposal_withdrawal_key(proposal_id).into(), key)
            .await
    }

    /// Get the state of a proposal.
    async fn proposal_state(&self, proposal_id: u64) -> Result<Option<proposal::State>> {
        Ok(self
            .get_domain::<proposal::State, _>(state_key::proposal_state(proposal_id).into())
            .await?)
    }

    /// Get all the unfinished proposal ids.
    async fn unfinished_proposals(&self) -> Result<BTreeSet<u64>> {
        Ok(self
            .get_domain::<proposal::ProposalList, _>(state_key::unfinished_proposals().into())
            .await?
            .unwrap_or_default()
            .proposals)
    }

    /// Set the state of a proposal.
    async fn put_proposal_state(&self, proposal_id: u64, state: proposal::State) -> Result<()> {
        // Set the state of the proposal
        self.put_domain(state_key::proposal_state(proposal_id).into(), state)
            .await;

        // Track the index
        let mut unfinished_proposals = self
            .get_domain::<proposal::ProposalList, _>(state_key::unfinished_proposals().into())
            .await?
            .unwrap_or_default();
        match state {
            proposal::State::Voting | proposal::State::Withdrawn => {
                // If we're setting the proposal to a non-finished state, track it in our list of
                // proposals that are not finished
                unfinished_proposals.proposals.insert(proposal_id);
            }
            proposal::State::Finished { .. } => {
                // If we're setting the proposal to a finished state, remove it from our list of
                // proposals that are not finished
                unfinished_proposals.proposals.remove(&proposal_id);
            }
        }

        // Put the modified list back into the state
        self.put_domain(
            state_key::unfinished_proposals().into(),
            unfinished_proposals,
        )
        .await;

        Ok(())
    }

    /// Get the withdrawal key of a proposal.
    async fn proposal_withdrawal_key(
        &self,
        proposal_id: u64,
    ) -> Result<Option<VerificationKey<SpendAuth>>> {
        Ok(self
            .get_domain::<VerificationKey<SpendAuth>, _>(
                state_key::proposal_withdrawal_key(proposal_id).into(),
            )
            .await?)
    }

    /// Get the vote of a validator on a particular proposal.
    async fn validator_vote(
        &self,
        proposal_id: u64,
        identity_key: IdentityKey,
    ) -> Result<Option<Vote>> {
        Ok(self
            .get_domain::<Vote, _>(state_key::validator_vote(proposal_id, identity_key).into())
            .await?)
    }

    /// Get the proposal voting end block for a given proposal.
    async fn proposal_voting_end(&self, proposal_id: u64) -> Result<Option<u64>> {
        Ok(self
            .get_proto::<u64>(state_key::proposal_voting_end(proposal_id).into())
            .await?)
    }

    /// Set the proposal voting end block height for a proposal.
    async fn put_proposal_voting_end(&self, proposal_id: u64, end_block: u64) {
        self.put_proto(
            state_key::proposal_voting_end(proposal_id).into(),
            end_block,
        )
        .await
    }
}

impl<T: StateExt> View for T {}
