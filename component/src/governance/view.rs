use std::{collections::BTreeSet, pin::Pin, str::FromStr};

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use penumbra_chain::StateReadExt as _;
use penumbra_crypto::{
    asset::Amount,
    stake::{DelegationToken, IdentityKey},
    GovernanceKey, Nullifier, Value, STAKING_TOKEN_DENOM,
};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};
use penumbra_tct as tct;
use penumbra_transaction::action::{Proposal, ProposalPayload, Vote};
use tokio::task::JoinSet;

use crate::{
    shielded_pool::{StateReadExt as _, StateWriteExt as _, SupplyRead},
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

    /// Get the proposal voting end block for a given proposal.
    async fn proposal_voting_start_position(
        &self,
        proposal_id: u64,
    ) -> Result<Option<tct::Position>> {
        Ok(self
            .get_proto::<u64>(&state_key::proposal_voting_start_position(proposal_id))
            .await?
            .map(Into::into))
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
    async fn check_nullifier_unvoted_for_proposal(
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

    /// Throw an error if the proposal is not voteable.
    async fn check_proposal_voteable(&self, proposal_id: u64) -> Result<()> {
        if let Some(proposal_state) = self.proposal_state(proposal_id).await? {
            use proposal::State::*;
            match proposal_state {
                Voting => {
                    // This is when you can vote on a proposal
                }
                Withdrawn { .. } => {
                    anyhow::bail!("proposal {} has already been withdrawn", proposal_id)
                }
                Finished { .. } | Claimed { .. } => {
                    anyhow::bail!("voting on proposal {} has already concluded", proposal_id)
                }
            }
        } else {
            anyhow::bail!("proposal {} does not exist", proposal_id);
        }

        Ok(())
    }

    /// Throw an error if the proposal was not started at the claimed position.
    async fn check_proposal_started_at_position(
        &self,
        proposal_id: u64,
        claimed_position: tct::Position,
    ) -> Result<()> {
        if let Some(position) = self.proposal_voting_start_position(proposal_id).await? {
            if position != claimed_position {
                anyhow::bail!(
                    "proposal {} was not started at claimed start position of {:?}",
                    proposal_id,
                    claimed_position
                );
            }
        } else {
            anyhow::bail!("proposal {} does not exist", proposal_id);
        }

        Ok(())
    }

    /// Throw an error if the nullifier was spent before the proposal started.
    async fn check_nullifier_unspent_before_start_block_height(
        &self,
        proposal_id: u64,
        nullifier: &Nullifier,
    ) -> Result<()> {
        let Some(start_height) = self.proposal_voting_start(proposal_id).await? else {
            anyhow::bail!("proposal {} does not exist", proposal_id);
        };

        if let Some(spend_info) = self.spend_info(*nullifier).await? {
            if spend_info.spend_height < start_height {
                anyhow::bail!(
                    "nullifier {} was already spent at block height {} before proposal started at block height {}",
                    nullifier,
                    spend_info.spend_height,
                    start_height
                );
            }
        }

        Ok(())
    }

    /// Throw an error if the exchange between the value and the unbonded amount isn't correct for
    /// the proposal given.
    async fn check_unbonded_amount_correct_exchange_for_proposal(
        &self,
        proposal_id: u64,
        value: &Value,
        unbonded_amount: &Amount,
    ) -> Result<()> {
        // Attempt to find the denom for the asset ID of the specified value
        let Some(denom) = self.denom_by_asset(&value.asset_id).await? else {
            anyhow::bail!("unknown asset id {} is not a delegation token", value.asset_id);
        };

        // Attempt to find the validator identity for the specified denom, failing if it is not a
        // delegation token
        let validator_identity = DelegationToken::try_from(denom)?.validator();

        // Attempt to look up the snapshotted `RateData` for the validator at the start of the proposal
        let Some(rate_data) = self
            .rate_data_at_proposal_start(proposal_id, validator_identity)
            .await? else {
                anyhow::bail!("validator {} was not active at the start of proposal {}", validator_identity, proposal_id);
            };

        // Check that the unbonded amount is correct relative to that exchange rate
        if rate_data.unbonded_amount(value.amount.into()) != u64::from(*unbonded_amount) {
            anyhow::bail!(
                "unbonded amount {}{} does not correspond to {} staked delegation tokens for validator {} using the exchange rate at the start of proposal {}",
                unbonded_amount,
                *STAKING_TOKEN_DENOM,
                value.amount,
                validator_identity,
                proposal_id,
            );
        }

        Ok(())
    }

    async fn check_height_in_future_of_voting_end(&self, height: u64) -> Result<()> {
        let block_height = self.get_block_height().await?;
        let voting_blocks = self.get_chain_params().await?.proposal_voting_blocks;
        let voting_end_height = block_height + voting_blocks;

        if height < voting_end_height {
            anyhow::bail!(
                "effective height {} is less than the block height {} for the end of the voting period",
                height,
                voting_end_height
            );
        }
        Ok(())
    }

    /// Check that the validator has not voted on the proposal.
    async fn check_validator_has_not_voted(
        &self,
        proposal_id: u64,
        identity_key: &IdentityKey,
    ) -> Result<()> {
        if let Some(_vote) = self.validator_vote(proposal_id, *identity_key).await? {
            anyhow::bail!(
                "validator {} has already voted on proposal {}",
                identity_key,
                proposal_id
            );
        }

        Ok(())
    }

    /// Check that the governance key matches the validator's identity key.
    async fn check_governance_key_matches_validator(
        &self,
        identity_key: &IdentityKey,
        governance_key: &GovernanceKey,
    ) -> Result<()> {
        if let Some(validator) = self.validator(identity_key).await? {
            if validator.governance_key != *governance_key {
                anyhow::bail!(
                    "governance key {} does not match validator {}",
                    governance_key,
                    identity_key
                );
            }
        } else {
            anyhow::bail!("validator {} does not exist", identity_key);
        }

        Ok(())
    }

    /// Check that a deposit claim could be made on the proposal.
    async fn check_proposal_claimable(&self, proposal_id: u64) -> Result<()> {
        if let Some(proposal_state) = self.proposal_state(proposal_id).await? {
            use proposal::State::*;
            match proposal_state {
                Voting => {
                    anyhow::bail!("proposal {} is still voting", proposal_id)
                }
                Withdrawn { .. } => {
                    anyhow::bail!(
                        "proposal {} has been withdrawn but voting has not concluded",
                        proposal_id
                    )
                }
                Finished { .. } => {
                    // This is when you can claim a proposal
                }
                Claimed { .. } => {
                    anyhow::bail!(
                        "the deposit for proposal {} has already been claimed",
                        proposal_id
                    )
                }
            }
        } else {
            anyhow::bail!("proposal {} does not exist", proposal_id);
        }

        Ok(())
    }

    /// Check that the deposit claim amount matches the proposal's deposit amount.
    async fn check_proposal_claim_valid_deposit(
        &self,
        proposal_id: u64,
        claim_deposit_amount: Amount,
    ) -> Result<()> {
        if let Some(proposal_deposit_amount) = self.proposal_deposit_amount(proposal_id).await? {
            if claim_deposit_amount != proposal_deposit_amount {
                anyhow::bail!(
                    "proposal deposit claim for {}{} does not match proposal deposit of {}{}",
                    claim_deposit_amount,
                    *STAKING_TOKEN_DENOM,
                    proposal_deposit_amount,
                    *STAKING_TOKEN_DENOM,
                );
            }
        } else {
            anyhow::bail!("proposal {} does not exist", proposal_id);
        }

        Ok(())
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

        // Snapshot the rate data and voting power for all active validators at this height
        let mut js = JoinSet::new();
        for identity_key in self.validator_identity_list().await? {
            let state = self.validator_state(&identity_key);
            let rate_data = self.current_validator_rate(&identity_key);
            let power = self.validator_power(&identity_key);
            js.spawn(async move {
                let state = state
                    .await?
                    .expect("every known validator must have a recorded state");
                // Compute the rate data, only for active validators, and write it to the state
                let per_validator = if state == validator::State::Active {
                    let rate_data = rate_data
                        .await?
                        .expect("every known validator must have a recorded current rate");
                    let power = power
                        .await?
                        .expect("every known validator must have a recorded current power");
                    Some((identity_key, rate_data, power))
                } else {
                    None
                };
                // Return the pair, to be written to the state
                Ok::<_, anyhow::Error>(per_validator)
            });
        }
        // Iterate over all the futures and insert them into the state (this can be done in
        // arbitrary order, because they are non-overlapping)
        while let Some(per_validator) = js.join_next().await.transpose()? {
            if let Some((identity_key, rate_data, power)) = per_validator? {
                self.put(
                    state_key::rate_data_at_proposal_start(proposal_id, identity_key),
                    rate_data,
                );
                self.put_proto(
                    state_key::voting_power_at_proposal_start(proposal_id, identity_key),
                    power,
                )
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

    /// Set the proposal voting start block height for a proposal.
    async fn put_proposal_voting_start(&mut self, proposal_id: u64, end_block: u64) {
        self.put_proto(state_key::proposal_voting_start(proposal_id), end_block);
    }

    /// Set the proposal voting end block height for a proposal.
    async fn put_proposal_voting_end(&mut self, proposal_id: u64, end_block: u64) {
        self.put_proto(state_key::proposal_voting_end(proposal_id), end_block);
    }

    /// Set the proposal voting start position for a proposal.
    async fn put_proposal_voting_start_position(
        &mut self,
        proposal_id: u64,
        start_position: tct::Position,
    ) {
        self.put_proto(
            state_key::proposal_voting_start_position(proposal_id),
            u64::from(start_position),
        );
    }

    /// Mark a nullifier as having voted on a proposal.
    async fn mark_nullifier_voted_on_proposal(
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

    /// Record a delegator vote on a proposal.
    async fn cast_delegator_vote(
        &mut self,
        proposal_id: u64,
        vote: Vote,
        nullifier: &Nullifier,
        unbonded_amount: Amount,
    ) -> Result<()> {
        // Record the vote
        self.put(
            state_key::delegator_vote(proposal_id, vote, nullifier),
            unbonded_amount,
        );

        Ok(())
    }
}

impl<T: StateWrite + StateReadExt> StateWriteExt for T {}
