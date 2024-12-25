use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use futures::StreamExt;
use ibc_types::core::client::ClientId;
use penumbra_sdk_asset::{asset, Value, STAKING_TOKEN_DENOM};
use penumbra_sdk_ibc::component::ClientStateReadExt as _;
use penumbra_sdk_ibc::component::ClientStateWriteExt as _;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};
use penumbra_sdk_sct::{
    component::{clock::EpochRead, tree::SctRead},
    Nullifier,
};
use penumbra_sdk_shielded_pool::component::AssetRegistryRead;
use penumbra_sdk_stake::{
    component::{validator_handler::ValidatorDataRead, ConsensusIndexRead},
    DelegationToken, GovernanceKey, IdentityKey,
};
use penumbra_sdk_tct as tct;
use tokio::task::JoinSet;
use tracing::instrument;

use penumbra_sdk_stake::{rate::RateData, validator};

use crate::{
    change::ParameterChange,
    params::GovernanceParameters,
    proposal::{Proposal, ProposalPayload},
    proposal_state::State as ProposalState,
    state_key::persistent_flags,
    validator_vote::action::ValidatorVoteReason,
    vote::Vote,
};
use crate::{state_key, tally::Tally};

#[async_trait]
pub trait StateReadExt: StateRead + penumbra_sdk_stake::StateReadExt {
    /// Returns true if the next height is an upgrade height.
    /// We look-ahead to the next height because we want to halt the chain immediately after
    /// committing the block.
    async fn is_pre_upgrade_height(&self) -> Result<bool> {
        let Some(next_upgrade_height) = self
            .nonverifiable_get_raw(state_key::upgrades::next_upgrade().as_bytes())
            .await?
        else {
            return Ok(false);
        };

        let next_upgrade_height = u64::from_be_bytes(next_upgrade_height.as_slice().try_into()?);

        let current_height = self.get_block_height().await?;
        Ok(current_height.saturating_add(1) == next_upgrade_height)
    }

    /// Gets the governance parameters from the JMT.
    async fn get_governance_params(&self) -> Result<GovernanceParameters> {
        self.get(state_key::governance_params())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Missing GovernanceParameters"))
    }

    /// Get the id of the next proposal in the sequence of ids.
    async fn next_proposal_id(&self) -> Result<u64> {
        Ok(self
            .get_proto::<u64>(state_key::next_proposal_id())
            .await?
            .unwrap_or_default())
    }

    /// Get the proposal definition for a proposal.
    async fn proposal_definition(&self, proposal_id: u64) -> Result<Option<Proposal>> {
        Ok(self
            .get(&state_key::proposal_definition(proposal_id))
            .await?)
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
    async fn proposal_state(&self, proposal_id: u64) -> Result<Option<ProposalState>> {
        Ok(self
            .get::<ProposalState>(&state_key::proposal_state(proposal_id))
            .await?)
    }

    /// Get all the unfinished proposal ids.
    async fn unfinished_proposals(&self) -> Result<BTreeSet<u64>> {
        let prefix = state_key::all_unfinished_proposals();
        let mut stream = self.prefix_proto(prefix);
        let mut proposals = BTreeSet::new();
        while let Some((key, ())) = stream.next().await.transpose()? {
            let proposal_id = u64::from_str(
                key.rsplit('/')
                    .next()
                    .context("invalid key for unfinished proposal")?,
            )?;
            proposals.insert(proposal_id);
        }
        Ok(proposals)
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

    /// Get the proposal voting start block for a given proposal.
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

    /// Get the proposal voting start block for a given proposal.
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
    async fn total_voting_power_at_proposal_start(&self, proposal_id: u64) -> Result<u64> {
        Ok(self
            .validator_voting_power_at_proposal_start(proposal_id)
            .await?
            .values()
            .copied()
            .sum())
    }

    /// Check whether a nullifier was spent for a given proposal.
    async fn check_nullifier_unvoted_for_proposal(
        &self,
        proposal_id: u64,
        nullifier: &Nullifier,
    ) -> Result<()> {
        if let Some(height) = self
            .get_proto::<u64>(&state_key::voted_nullifier_lookup_for_proposal(
                proposal_id,
                nullifier,
            ))
            .await?
        {
            // If the nullifier was already voted with, error:
            anyhow::bail!(
                "nullifier {nullifier} was already used for voting on proposal {proposal_id} at height {height}",
            );
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

    /// Throw an error if the proposal is not votable.
    async fn check_proposal_votable(&self, proposal_id: u64) -> Result<()> {
        if let Some(proposal_state) = self.proposal_state(proposal_id).await? {
            use crate::proposal_state::State::*;
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

    /// Look up the validator for a given asset ID, if it is a delegation token.
    async fn validator_by_delegation_asset(&self, asset_id: asset::Id) -> Result<IdentityKey> {
        // Attempt to find the denom for the asset ID of the specified value
        let Some(denom_metadata) = self.denom_metadata_by_asset(&asset_id).await else {
            anyhow::bail!("asset ID {} does not correspond to a known denom", asset_id);
        };

        // Attempt to find the validator identity for the specified denom, failing if it is not a
        // delegation token
        let validator_identity = DelegationToken::try_from(denom_metadata)?.validator();

        Ok(validator_identity)
    }

    /// Throw an error if the exchange between the value and the unbonded amount isn't correct for
    /// the proposal given.
    async fn check_unbonded_amount_correct_exchange_for_proposal(
        &self,
        proposal_id: u64,
        value: &Value,
        unbonded_amount: &Amount,
    ) -> Result<()> {
        let validator_identity = self.validator_by_delegation_asset(value.asset_id).await?;

        // Attempt to look up the snapshotted `RateData` for the validator at the start of the proposal
        let Some(rate_data) = self
            .rate_data_at_proposal_start(proposal_id, validator_identity)
            .await?
        else {
            anyhow::bail!(
                "validator {} was not active at the start of proposal {}",
                validator_identity,
                proposal_id
            );
        };

        // Check that the unbonded amount is correct relative to that exchange rate
        if rate_data.unbonded_amount(value.amount).value() != unbonded_amount.value() {
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
        let voting_blocks = self.get_governance_params().await?.proposal_voting_blocks;
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
        if let Some(validator) = self.get_validator_definition(identity_key).await? {
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
            match proposal_state {
                ProposalState::Voting => {
                    anyhow::bail!("proposal {} is still voting", proposal_id)
                }
                ProposalState::Withdrawn { .. } => {
                    anyhow::bail!(
                        "proposal {} has been withdrawn but voting has not concluded",
                        proposal_id
                    )
                }
                ProposalState::Finished { .. } => {
                    // This is when you can claim a proposal
                }
                ProposalState::Claimed { .. } => {
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

    /// Get a specific validator's voting power for a proposal.
    async fn specific_validator_voting_power_at_proposal_start(
        &self,
        proposal_id: u64,
        identity_key: IdentityKey,
    ) -> Result<u64> {
        self.get_proto(&state_key::voting_power_at_proposal_start(
            proposal_id,
            identity_key,
        ))
        .await
        .map(Option::unwrap_or_default)
    }

    /// Get all the active validator voting power for the proposal.
    async fn validator_voting_power_at_proposal_start(
        &self,
        proposal_id: u64,
    ) -> Result<BTreeMap<IdentityKey, u64>> {
        let mut powers = BTreeMap::new();

        let prefix = state_key::all_voting_power_at_proposal_start(proposal_id);
        let mut stream = self.prefix_proto(&prefix);

        while let Some((key, power)) = stream.next().await.transpose()? {
            let identity_key = key
                .rsplit('/')
                .next()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "incorrect key format for validator voting power at proposal start"
                    )
                })?
                .parse()?;
            powers.insert(identity_key, power);
        }

        Ok(powers)
    }

    /// Check whether a validator was active at the start of a proposal, and fail if not.
    async fn check_validator_active_at_proposal_start(
        &self,
        proposal_id: u64,
        identity_key: &IdentityKey,
    ) -> Result<()> {
        if self
            .get_proto::<u64>(&state_key::voting_power_at_proposal_start(
                proposal_id,
                *identity_key,
            ))
            .await?
            .is_none()
        {
            anyhow::bail!(
                "validator {} was not active at the start of proposal {}",
                identity_key,
                proposal_id
            );
        }

        Ok(())
    }

    /// Get all the validator votes for the proposal.
    async fn validator_votes(&self, proposal_id: u64) -> Result<BTreeMap<IdentityKey, Vote>> {
        let mut votes = BTreeMap::new();

        let prefix = state_key::all_validator_votes_for_proposal(proposal_id);
        let mut stream = self.prefix(&prefix);

        while let Some((key, vote)) = stream.next().await.transpose()? {
            let identity_key = key
                .rsplit('/')
                .next()
                .ok_or_else(|| anyhow::anyhow!("incorrect key format for validator vote"))?
                .parse()?;
            votes.insert(identity_key, vote);
        }

        Ok(votes)
    }

    /// Get all the *tallied* delegator votes for the proposal (excluding those which have been
    /// cast but not tallied).
    async fn tallied_delegator_votes(
        &self,
        proposal_id: u64,
    ) -> Result<BTreeMap<IdentityKey, Tally>> {
        let mut tallies = BTreeMap::new();

        let prefix = state_key::all_tallied_delegator_votes_for_proposal(proposal_id);
        let mut stream = self.prefix(&prefix);

        while let Some((key, tally)) = stream.next().await.transpose()? {
            let identity_key = key
                .rsplit('/')
                .next()
                .ok_or_else(|| anyhow::anyhow!("incorrect key format for delegator vote tally"))?
                .parse()?;
            tallies.insert(identity_key, tally);
        }

        Ok(tallies)
    }

    /// Add up all the currently tallied votes (without tallying any cast votes that haven't been
    /// tallied yet).
    async fn current_tally(&self, proposal_id: u64) -> Result<Tally> {
        let validator_powers = self
            .validator_voting_power_at_proposal_start(proposal_id)
            .await?;
        let mut validator_votes = self.validator_votes(proposal_id).await?;
        let mut delegator_tallies = self.tallied_delegator_votes(proposal_id).await?;

        // For each validator, tally their own vote, overriding it with any tallied delegator votes
        let mut tally = Tally::default();
        for (validator, power) in validator_powers.into_iter() {
            let delegator_tally = delegator_tallies.remove(&validator).unwrap_or_default();
            if let Some(vote) = validator_votes.remove(&validator) {
                // The effective power of a validator is the voting power of that validator at
                // proposal start, minus the total voting power used by delegators to that validator
                // who have voted. Their votes will be added back in below, re-assigning their
                // voting power to their chosen votes.
                let effective_power = power.saturating_sub(delegator_tally.total());
                tally += (vote, effective_power).into();
            }
            // Add the delegator votes in, regardless of if the validator has voted.
            tally += delegator_tally;
        }

        assert!(
            validator_votes.is_empty(),
            "no inactive validator should have voted"
        );
        assert!(
            delegator_tallies.is_empty(),
            "no delegator should have been able to vote for an inactive validator"
        );

        Ok(tally)
    }

    /// Gets the parameter changes scheduled for the given height, if any.
    async fn param_changes_for_height(&self, height: u64) -> Result<Option<ParameterChange>> {
        self.get(&state_key::param_changes_for_height(height)).await
    }

    /// Check if any proposal is started in this block.
    fn proposal_started(&self) -> bool {
        self.object_get::<()>(state_key::proposal_started())
            .is_some()
    }

    async fn is_chain_halted(&self) -> bool {
        self.nonverifiable_get_proto(state_key::persistent_flags::halt_bit().as_bytes())
            .await
            .expect("no deserialization errors")
            .unwrap_or_default()
    }
}

impl<T: StateRead + penumbra_sdk_stake::StateReadExt + ?Sized> StateReadExt for T {}

#[async_trait]
pub trait StateWriteExt: StateWrite + penumbra_sdk_ibc::component::ConnectionStateWriteExt {
    /// Writes the provided governance parameters to the JMT.
    fn put_governance_params(&mut self, params: GovernanceParameters) {
        // Change the governance parameters:
        self.put(state_key::governance_params().into(), params)
    }

    /// Initialize the proposal counter so that it can always be read.
    fn init_proposal_counter(&mut self) {
        self.put_proto(state_key::next_proposal_id().to_string(), 0);
    }

    /// Store a new proposal with a new proposal id.
    async fn new_proposal(&mut self, proposal: &Proposal) -> Result<u64> {
        let proposal_id = self.next_proposal_id().await?;
        if proposal_id != proposal.id {
            anyhow::bail!(
                "proposal id {} does not match next proposal id {}",
                proposal.id,
                proposal_id
            );
        }

        // Snapshot the rate data and voting power for all active validators at this height
        let mut js = JoinSet::new();
        let mut validator_identity_stream = self.consensus_set_stream()?;
        while let Some(identity_key) = validator_identity_stream.next().await {
            let identity_key = identity_key?;

            let state = self.get_validator_state(&identity_key);
            let rate_data = self.get_validator_rate(&identity_key);
            let power: penumbra_sdk_proto::state::future::DomainFuture<
                Amount,
                <Self as StateRead>::GetRawFut,
            > = self.get_validator_power(&identity_key);
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
                anyhow::Ok(per_validator)
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
                self.put(
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
    async fn mark_nullifier_voted(&mut self, proposal_id: u64, nullifier: &Nullifier) {
        self.put_proto(
            state_key::voted_nullifier_lookup_for_proposal(proposal_id, nullifier),
            self.get_block_height()
                .await
                .expect("block height should be set"),
        );
    }

    /// Record in the object store that some proposal has started.
    fn mark_proposal_started(&mut self) {
        self.object_put(state_key::proposal_started(), ());
    }

    /// Store the proposal deposit amount.
    fn put_deposit_amount(&mut self, proposal_id: u64, amount: Amount) {
        self.put(state_key::proposal_deposit_amount(proposal_id), amount);
    }

    /// Set the state of a proposal.
    fn put_proposal_state(&mut self, proposal_id: u64, state: ProposalState) {
        // Set the state of the proposal
        self.put(state_key::proposal_state(proposal_id), state.clone());

        match &state {
            ProposalState::Voting | ProposalState::Withdrawn { .. } => {
                // If we're setting the proposal to a non-finished state, track it in our list of
                // proposals that are not finished
                self.put_proto(state_key::unfinished_proposal(proposal_id), ());
            }
            ProposalState::Finished { .. } | ProposalState::Claimed { .. } => {
                // If we're setting the proposal to a finished or claimed state, remove it from our list of
                // proposals that are not finished
                self.delete(state_key::unfinished_proposal(proposal_id));
            }
        }
    }

    /// Record a validator vote for a proposal.
    fn cast_validator_vote(
        &mut self,
        proposal_id: u64,
        identity_key: IdentityKey,
        vote: Vote,
        reason: ValidatorVoteReason,
    ) {
        // Record the vote
        self.put(state_key::validator_vote(proposal_id, identity_key), vote);
        // Record the vote justification
        self.put(
            state_key::validator_vote_reason(proposal_id, identity_key),
            reason,
        );
    }

    /// Set the proposal voting start block height for a proposal.
    fn put_proposal_voting_start(&mut self, proposal_id: u64, end_block: u64) {
        self.put_proto(state_key::proposal_voting_start(proposal_id), end_block);
    }

    /// Set the proposal voting end block height for a proposal.
    fn put_proposal_voting_end(&mut self, proposal_id: u64, end_block: u64) {
        self.put_proto(state_key::proposal_voting_end(proposal_id), end_block);
    }

    /// Set the proposal voting start position for a proposal.
    fn put_proposal_voting_start_position(
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
    async fn mark_nullifier_voted_on_proposal(&mut self, proposal_id: u64, nullifier: &Nullifier) {
        self.put_proto(
            state_key::voted_nullifier_lookup_for_proposal(proposal_id, nullifier),
            self.get_block_height()
                .await
                .expect("block height should be set"),
        );
    }

    /// Record a delegator vote on a proposal.
    async fn cast_delegator_vote(
        &mut self,
        proposal_id: u64,
        identity_key: IdentityKey,
        vote: Vote,
        nullifier: &Nullifier,
        unbonded_amount: Amount,
    ) -> Result<()> {
        // Convert the unbonded amount into voting power
        let power = unbonded_amount.value() as u64;
        let tally: Tally = (vote, power).into();

        // Record the vote
        self.put(
            state_key::untallied_delegator_vote(proposal_id, identity_key, nullifier),
            tally,
        );

        Ok(())
    }

    /// Tally delegator votes by sweeping them into the aggregate for each validator, for each proposal.
    #[instrument(skip(self))]
    async fn tally_delegator_votes(&mut self, just_for_proposal: Option<u64>) -> Result<()> {
        // Iterate over all the delegator votes, or just the ones for a specific proposal
        let prefix = if let Some(proposal_id) = just_for_proposal {
            state_key::all_untallied_delegator_votes_for_proposal(proposal_id)
        } else {
            state_key::all_untallied_delegator_votes().to_string()
        };
        let mut prefix_stream = self.prefix(&prefix);

        // We need to keep track of modifications and then apply them after iteration, because
        // `self.prefix(..)` borrows `self` immutably, so we can't mutate `self` during iteration
        let mut keys_to_delete = vec![];
        let mut new_tallies: BTreeMap<u64, BTreeMap<IdentityKey, Tally>> = BTreeMap::new();

        while let Some((key, tally)) = prefix_stream.next().await.transpose()? {
            // Extract the validator identity key from the key string
            let mut reverse_path_elements = key.rsplit('/');
            reverse_path_elements.next(); // skip the nullifier element of the key
            let identity_key = reverse_path_elements
                .next()
                .ok_or_else(|| {
                    anyhow::anyhow!("unexpected key format for untallied delegator vote")
                })?
                .parse()?;
            let proposal_id = reverse_path_elements
                .next()
                .ok_or_else(|| {
                    anyhow::anyhow!("unexpected key format for untallied delegator vote")
                })?
                .parse()?;

            // Get the current tally for this validator
            let mut current_tally = self
                .get::<Tally>(&state_key::tallied_delegator_votes(
                    proposal_id,
                    identity_key,
                ))
                .await?
                .unwrap_or_default();

            // Add the new tally to the current tally
            current_tally += tally;

            // Remember the new tally
            new_tallies
                .entry(proposal_id)
                .or_default()
                .insert(identity_key, current_tally);

            // Remember to delete this key
            keys_to_delete.push(key);
        }

        // Explicit drop because we need to borrow self mutably again below
        drop(prefix_stream);

        // Actually record the key deletions in the state
        for key in keys_to_delete {
            self.delete(key);
        }

        // Actually record the new tallies in the state
        for (proposal_id, new_tallies_for_proposal) in new_tallies {
            for (identity_key, tally) in new_tallies_for_proposal {
                tracing::debug!(
                    proposal_id,
                    identity_key = %identity_key,
                    yes = %tally.yes(),
                    no = %tally.no(),
                    abstain = %tally.abstain(),
                    "tallying delegator votes"
                );
                self.put(
                    state_key::tallied_delegator_votes(proposal_id, identity_key),
                    tally,
                );
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn enact_proposal(
        &mut self,
        proposal_id: u64,
        payload: &ProposalPayload,
    ) -> Result<Result<()>> // inner error from proposal execution
    {
        match payload {
            ProposalPayload::Signaling { .. } => {
                // Nothing to do for signaling proposals
                tracing::info!("signaling proposal passed, nothing to do");
            }
            ProposalPayload::Emergency { halt_chain } => {
                // If the proposal calls to halt the chain...
                if *halt_chain {
                    // Print an informational message and signal to the consensus worker to halt the
                    // process after the state is committed
                    tracing::info!("emergency proposal passed calling for immediate chain halt");
                    self.signal_halt();
                }
            }
            ProposalPayload::ParameterChange(change) => {
                let current_height = self.get_block_height().await?;
                // The parameter change should take effect in the next block.
                let change_height = current_height + 1;
                tracing::info!(
                    change_height,
                    ?change,
                    "parameter change proposal passed, scheduling for next block"
                );
                // Note: if two parameter changes are scheduled for the same height by two
                // passing proposals that were submitted in exactly the same block, one will
                // clobber the other. This is undesirable but seems unlikely to happen in practice.
                self.put(
                    state_key::param_changes_for_height(change_height),
                    change.clone(),
                );
            }
            ProposalPayload::CommunityPoolSpend {
                transaction_plan: _,
            } => {
                // All we need to do here is signal to the `App` that we'd like this transaction to
                // be slotted in at the end of the block:
                self.deliver_community_pool_transaction(proposal_id).await?;
            }
            ProposalPayload::UpgradePlan { height } => {
                tracing::info!(target_height = height, "upgrade plan proposal passed");
                self.signal_upgrade(*height).await?;
            }
            ProposalPayload::FreezeIbcClient { client_id } => {
                let client_id = &ClientId::from_str(client_id)
                    .map_err(|e| tonic::Status::aborted(format!("invalid client id: {e}")))?;
                let client_state = self.get_client_state(client_id).await?;

                let frozen_client =
                    client_state.with_frozen_height(ibc_types::core::client::Height {
                        revision_number: 0,
                        revision_height: 1,
                    });
                self.put_client(client_id, frozen_client);
            }
            ProposalPayload::UnfreezeIbcClient { client_id } => {
                let client_id = &ClientId::from_str(client_id)
                    .map_err(|e| tonic::Status::aborted(format!("invalid client id: {e}")))?;
                let client_state = self.get_client_state(client_id).await?;

                let unfrozen_client = client_state.unfrozen();
                self.put_client(client_id, unfrozen_client);
            }
        }
        Ok(Ok(()))
    }

    async fn deliver_community_pool_transaction(&mut self, proposal: u64) -> Result<()> {
        // Schedule for beginning of next block
        let delivery_height = self.get_block_height().await? + 1;

        tracing::info!(%proposal, %delivery_height, "scheduling Community Pool transaction for delivery at next block");

        self.put_proto(
            state_key::deliver_single_community_pool_transaction_at_height(
                delivery_height,
                proposal,
            ),
            proposal,
        );
        Ok(())
    }

    /// Records the next upgrade height.
    /// After commititng the height, the chain should halt and wait for an upgrade.
    /// It re-uses the same mechanism as emergency halting that prevents the chain from
    /// restarting, without setting `halt_bit`.
    async fn signal_upgrade(&mut self, height: u64) -> Result<()> {
        self.nonverifiable_put_raw(
            state_key::upgrades::next_upgrade().into(),
            height.to_be_bytes().to_vec(),
        );
        Ok(())
    }

    /// Sets the application `halt_bit` to `true`, signaling that
    /// the chain should be halted, and preventing restarts until
    /// a migration is ran.
    fn signal_halt(&mut self) {
        self.nonverifiable_put_proto(persistent_flags::halt_bit().as_bytes().to_vec(), true);
    }

    /// Sets the application `halt_bit` to `false`, signaling that
    /// the chain can resume, and the application is ready to start.
    fn ready_to_start(&mut self) {
        self.nonverifiable_put_proto(persistent_flags::halt_bit().as_bytes().to_vec(), false);
    }
}

impl<T: StateWrite + StateReadExt + ?Sized> StateWriteExt for T {}
