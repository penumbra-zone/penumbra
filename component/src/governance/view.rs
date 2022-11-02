use std::collections::BTreeSet;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_crypto::{
    asset::Amount,
    rdsa::{SpendAuth, VerificationKey},
    Address, IdentityKey, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_storage2::{StateRead, StateWrite};
use penumbra_transaction::action::{Proposal, ProposalPayload, Vote};

use crate::stake::{self, validator, StateReadExt as _};

use super::{
    proposal::{self, ProposalList},
    state_key,
};

#[async_trait]
pub trait StateReadExt: StateRead {
    /// Get the id of the next proposal in the sequence of ids.
    async fn next_proposal_id(&self) -> Result<u64> {
        Ok(self
            .get_proto::<u64>(state_key::latest_proposal_id().into())
            .await?
            .map(|i| i + 1)
            .unwrap_or(0))
    }
    /// Get the proposal payload for a proposal.
    async fn proposal_payload(&self, proposal_id: u64) -> Result<Option<ProposalPayload>> {
        self.get_domain(state_key::proposal_payload(proposal_id).into())
            .await
    }
    /// Get the proposal deposit refund address for a proposal.
    async fn proposal_deposit_refund_address(&self, proposal_id: u64) -> Result<Option<Address>> {
        self.get_domain(state_key::proposal_deposit_refund_address(proposal_id).into())
            .await
    }

    /// Get the proposal deposit amount for a proposal.
    async fn proposal_deposit_amount(&self, proposal_id: u64) -> Result<Option<u64>> {
        self.get_proto(state_key::proposal_deposit_amount(proposal_id).into())
            .await
    }

    /// Get the proposals to be refunded in this block, along with their addresses and deposit
    /// amounts.
    ///
    /// This is meant to be called from within the shielded pool component, which will actually mint
    /// the notes.
    async fn proposal_refunds(&self, block_height: u64) -> Result<Vec<(u64, Address, Value)>> {
        let proposals = self
            .get_domain::<proposal::ProposalList, _>(
                state_key::proposal_refunds(block_height).into(),
            )
            .await?
            .unwrap_or_default()
            .proposals;

        let mut result = Vec::new();

        for proposal_id in proposals {
            let address = self
                .proposal_deposit_refund_address(proposal_id)
                .await?
                .expect("address must exist for proposal");
            let amount: Amount = self
                .get_domain(state_key::proposal_deposit_amount(proposal_id).into())
                .await?
                .expect("deposit amount must exist for proposal");
            result.push((
                proposal_id,
                address,
                Value {
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                    amount,
                },
            ));
        }

        Ok(result)
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

    /// Get the list of validators who voted on a proposal.
    async fn voting_validators(&self, proposal_id: u64) -> Result<Vec<IdentityKey>> {
        Ok(self
            .get_domain::<stake::validator::List, _>(
                state_key::voting_validators(proposal_id).into(),
            )
            .await?
            .unwrap_or_default()
            .0)
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
    async fn proposal_voting_start(&self, proposal_id: u64) -> Result<Option<u64>> {
        Ok(self
            .get_proto::<u64>(state_key::proposal_voting_start(proposal_id).into())
            .await?)
    }

    /// Get the proposal voting end block for a given proposal.
    async fn proposal_voting_end(&self, proposal_id: u64) -> Result<Option<u64>> {
        Ok(self
            .get_proto::<u64>(state_key::proposal_voting_end(proposal_id).into())
            .await?)
    }

    /// Get the total voting power across all validators.
    async fn total_voting_power(&self) -> Result<u64> {
        let mut total = 0;

        for identity_key in self.validator_list().await? {
            total += self
                .validator_power(&identity_key)
                .await?
                .unwrap_or_default();
        }

        Ok(total)
    }
}

impl<T: StateRead> StateReadExt for T {}

#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Store a new proposal with a new proposal id.
    async fn new_proposal(&self, proposal: &Proposal) -> Result<u64> {
        let proposal_id = self.next_proposal_id().await?;

        // Record this proposal id, so we won't re-use it
        self.put_proto(state_key::latest_proposal_id().into(), proposal_id)
            .await;

        // Store the proposal title
        self.put_proto(
            state_key::proposal_title(proposal_id).into(),
            proposal.title.clone(),
        )
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

        // Set the list of validators who have voted to the empty list
        self.put_domain(
            state_key::voting_validators(proposal_id).into(),
            validator::List::default(),
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

    /// Store the proposal deposit amount.
    async fn put_deposit_amount(&self, proposal_id: u64, amount: Amount) {
        self.put_domain(
            state_key::proposal_deposit_amount(proposal_id).into(),
            amount,
        )
        .await
    }

    /// Mark a proposal as to-be-refunded in this block.
    async fn add_proposal_refund(&self, block_height: u64, proposal_id: u64) -> Result<()> {
        let mut refunded_in_this_block = self
            .get_domain::<proposal::ProposalList, _>(
                state_key::proposal_refunds(block_height).into(),
            )
            .await?
            .unwrap_or_default();
        refunded_in_this_block.proposals.insert(proposal_id);
        self.put_domain(
            state_key::proposal_refunds(block_height).into(),
            refunded_in_this_block,
        )
        .await;
        Ok(())
    }
    /// Set all the unfinished proposal ids.
    async fn put_unfinished_proposals(&self, unfinished_proposals: ProposalList) {
        self.put_domain(
            state_key::unfinished_proposals().into(),
            unfinished_proposals,
        )
        .await;
    }

    /// Set the state of a proposal.
    async fn put_proposal_state(&self, proposal_id: u64, state: proposal::State) -> Result<()> {
        // Set the state of the proposal
        self.put_domain(state_key::proposal_state(proposal_id).into(), state.clone())
            .await;

        // Track the index
        let mut unfinished_proposals = self
            .get_domain::<proposal::ProposalList, _>(state_key::unfinished_proposals().into())
            .await?
            .unwrap_or_default();
        match &state {
            proposal::State::Voting | proposal::State::Withdrawn { .. } => {
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
        self.put_unfinished_proposals(unfinished_proposals).await;

        Ok(())
    }

    /// Record a validator vote for a proposal.
    async fn cast_validator_vote(&self, proposal_id: u64, identity_key: IdentityKey, vote: Vote) {
        // Record the vote
        self.put_domain(
            state_key::validator_vote(proposal_id, identity_key).into(),
            vote,
        )
        .await;

        // Record the fact that this validator has voted on this proposal
        let mut voting_validators = self
            .get_domain::<stake::validator::List, _>(
                state_key::voting_validators(proposal_id).into(),
            )
            .await
            .expect("can fetch voting validators")
            .unwrap_or_default();
        voting_validators.0.push(identity_key);
        self.put_domain(
            state_key::voting_validators(proposal_id).into(),
            voting_validators,
        )
        .await;
    }
    /// Set the proposal voting end block height for a proposal.
    async fn put_proposal_voting_start(&self, proposal_id: u64, end_block: u64) {
        self.put_proto(
            state_key::proposal_voting_start(proposal_id).into(),
            end_block,
        )
        .await
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

impl<T: StateWrite> StateWriteExt for T {}
