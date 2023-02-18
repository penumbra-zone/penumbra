use std::{collections::BTreeSet, pin::Pin, str::FromStr};

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use penumbra_chain::SpendInfo;
use penumbra_crypto::{asset::Amount, stake::IdentityKey, Nullifier};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::action::{Proposal, ProposalPayload, Vote};
use tokio::task::JoinSet;

use crate::{
    shielded_pool::StateWriteExt as _,
    stake::{rate::RateData, validator, StateReadExt as _},
};

use super::{
    proposal::{self, ProposalList},
    state_key,
};

#[async_trait]
pub trait StateReadExt: StateRead + crate::stake::StateReadExt {
    /// Get the id of the next proposal in the sequence of ids.
    async fn next_proposal_id(&self) -> Result<u64> {
        Ok(self
            .get_proto::<u64>(state_key::next_proposal_id())
            .await?
            .expect("counter is initialized"))
    }

    /// Get the proposal payload for a proposal.
    async fn proposal_payload(&self, proposal_id: u64) -> Result<Option<ProposalPayload>> {
        Ok(self
            .get(&state_key::proposal_definition(proposal_id))
            .await?
            .map(|p: Proposal| p.payload))
    }

    /// Get the proposal deposit amount for a proposal.
    async fn proposal_deposit_amount(&self, proposal_id: u64) -> Result<Option<Amount>> {
        self.get(&state_key::proposal_deposit_amount(proposal_id))
            .await
    }

    /// Get the state of a proposal.
    async fn proposal_state(&self, proposal_id: u64) -> Result<Option<proposal::State>> {
        Ok(self
            .get::<proposal::State>(&state_key::proposal_state(proposal_id))
            .await?)
    }

    /// Get all the unfinished proposal ids.
    async fn unfinished_proposals(&self) -> Result<BTreeSet<u64>> {
        Ok(self
            .get::<proposal::ProposalList>(state_key::unfinished_proposals())
            .await?
            .unwrap_or_default()
            .proposals)
    }

    /// Get the list of validators who voted on a proposal.
    async fn voting_validators(&self, proposal_id: u64) -> Result<Vec<IdentityKey>> {
        let k = state_key::voting_validators_list(proposal_id);
        let mut range: Pin<Box<dyn Stream<Item = Result<(String, Vote)>> + Send + '_>> =
            self.prefix(&k);

        range
            .next()
            .await
            .into_iter()
            .map(|r| IdentityKey::from_str(r?.0.rsplit('/').next().context("invalid key")?))
            .collect()
    }

    /// Get the vote of a validator on a particular proposal.
    async fn validator_vote(
        &self,
        proposal_id: u64,
        identity_key: IdentityKey,
    ) -> Result<Option<Vote>> {
        Ok(self
            .get::<Vote>(&state_key::validator_vote(proposal_id, identity_key))
            .await?)
    }

    /// Get the proposal voting end block for a given proposal.
    async fn proposal_voting_start(&self, proposal_id: u64) -> Result<Option<u64>> {
        Ok(self
            .get_proto::<u64>(&state_key::proposal_voting_start(proposal_id))
            .await?)
    }

    /// Get the proposal voting end block for a given proposal.
    async fn proposal_voting_end(&self, proposal_id: u64) -> Result<Option<u64>> {
        Ok(self
            .get_proto::<u64>(&state_key::proposal_voting_end(proposal_id))
            .await?)
    }

    /// Get the total voting power across all validators.
    async fn total_voting_power(&self) -> Result<u64> {
        let mut total = 0;

        for v in self.validator_list().await? {
            total += self
                .validator_power(&v.identity_key)
                .await?
                .unwrap_or_default();
        }

        Ok(total)
    }

    /// Check whether a nullifier was spent for a given proposal.
    async fn per_proposal_check_nullifier_unvoted(
        &self,
        proposal_id: u64,
        nullifier: &Nullifier,
    ) -> Result<()> {
        if let Some(height) = self
            .get_proto::<u64>(&state_key::per_proposal_voted_nullifier_lookup(
                proposal_id,
                nullifier,
            ))
            .await?
        {
            // If the nullifier was already voted with, error:
            return Err(anyhow::anyhow!(
                "nullifier {nullifier} was already used for voting on proposal {proposal_id} at height {height}",
            ));
        }

        Ok(())
    }

    /// Get the [`RateData`] for a validator at the start height of a given proposal.
    async fn rate_data_at_proposal_start(
        &self,
        proposal_id: u64,
        identity_key: IdentityKey,
    ) -> Result<Option<RateData>> {
        self.get(&state_key::rate_data_at_proposal_start(
            proposal_id,
            identity_key,
        ))
        .await
    }
}

impl<T: StateRead + crate::stake::StateReadExt + ?Sized> StateReadExt for T {}

#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Initialize the proposal counter at zero.
    async fn init_proposal_counter(&mut self) {
        self.put_proto(state_key::next_proposal_id().to_owned(), 0);
    }

    /// Store a new proposal with a new proposal id.
    async fn new_proposal(&mut self, proposal: &Proposal) -> Result<u64> {
        let proposal_id = self.next_proposal_id().await?;
        if proposal_id != proposal.id {
            return Err(anyhow::anyhow!(
                "proposal id {} does not match next proposal id {}",
                proposal.id,
                proposal_id
            ));
        }

        // Snapshot the rate data for all active validators at this height
        let mut js = JoinSet::new();
        for identity_key in self.validator_identity_list().await? {
            let state = self.validator_state(&identity_key);
            let rate_data = self.current_validator_rate(&identity_key);
            js.spawn(async move {
                let state = state
                    .await?
                    .expect("every known validator must have a recorded state");
                // Compute the rate data, only for active validators, and write it to the state
                let pair = if state == validator::State::Active {
                    let rate_data = rate_data
                        .await?
                        .expect("every known validator must have a recorded current rate");
                    Some((identity_key, rate_data))
                } else {
                    None
                };
                // Return the pair, to be written to the state
                Ok::<_, anyhow::Error>(pair)
            });
        }
        // Iterate over all the futures and insert them into the state (this can be done in
        // arbitrary order, because they are non-overlapping)
        while let Some(pair) = js.join_next().await.transpose()? {
            if let Some((identity_key, rate_data)) = pair? {
                self.put(
                    state_key::rate_data_at_proposal_start(proposal_id, identity_key),
                    rate_data,
                );
            }
        }

        // Record this proposal id, so we won't re-use it
        self.put_proto(state_key::next_proposal_id().to_owned(), proposal_id + 1);

        // Store the proposal data
        self.put(
            state_key::proposal_definition(proposal_id),
            proposal.clone(),
        );

        // Return the new proposal id
        Ok(proposal_id)
    }

    /// Mark a nullifier as spent for a given proposal.
    async fn mark_nullifier_voted(
        &mut self,
        proposal_id: u64,
        nullifier: &Nullifier,
    ) -> Result<()> {
        self.put_proto(
            state_key::per_proposal_voted_nullifier_lookup(proposal_id, nullifier),
            self.height().await,
        );

        Ok(())
    }

    /// Store the proposal deposit amount.
    async fn put_deposit_amount(&mut self, proposal_id: u64, amount: Amount) {
        self.put(state_key::proposal_deposit_amount(proposal_id), amount);
    }

    /// Set all the unfinished proposal ids.
    async fn put_unfinished_proposals(&mut self, unfinished_proposals: ProposalList) {
        self.put(
            state_key::unfinished_proposals().to_owned(),
            unfinished_proposals,
        );
    }

    /// Set the state of a proposal.
    async fn put_proposal_state(&mut self, proposal_id: u64, state: proposal::State) -> Result<()> {
        // Set the state of the proposal
        self.put(state_key::proposal_state(proposal_id), state.clone());

        // Track the index
        let mut unfinished_proposals = self
            .get::<proposal::ProposalList>(state_key::unfinished_proposals())
            .await?
            .unwrap_or_default();
        match &state {
            proposal::State::Voting | proposal::State::Withdrawn { .. } => {
                // If we're setting the proposal to a non-finished state, track it in our list of
                // proposals that are not finished
                unfinished_proposals.proposals.insert(proposal_id);
            }
            proposal::State::Finished { .. } | proposal::State::Claimed { .. } => {
                // If we're setting the proposal to a finished or claimed state, remove it from our list of
                // proposals that are not finished
                unfinished_proposals.proposals.remove(&proposal_id);
            }
        }

        // Put the modified list back into the state
        self.put_unfinished_proposals(unfinished_proposals).await;

        Ok(())
    }

    /// Record a validator vote for a proposal.
    async fn cast_validator_vote(
        &mut self,
        proposal_id: u64,
        identity_key: IdentityKey,
        vote: Vote,
    ) {
        // Record the vote
        self.put(state_key::validator_vote(proposal_id, identity_key), vote);
    }

    /// Set the proposal voting end block height for a proposal.
    async fn put_proposal_voting_start(&mut self, proposal_id: u64, end_block: u64) {
        self.put_proto(state_key::proposal_voting_start(proposal_id), end_block);
    }

    /// Set the proposal voting end block height for a proposal.
    async fn put_proposal_voting_end(&mut self, proposal_id: u64, end_block: u64) {
        self.put_proto(state_key::proposal_voting_end(proposal_id), end_block);
    }
}

impl<T: StateWrite + StateReadExt> StateWriteExt for T {}
