use anyhow::{Context as _, Result};

use super::proposal::{self, chain_params};
use penumbra_transaction::{
    action::{
        DelegatorVote, DelegatorVoteBody, Proposal, ProposalDepositClaim, ProposalSubmit,
        ProposalWithdraw, ValidatorVote, ValidatorVoteBody,
    },
    Transaction,
};

use std::sync::Arc;

use penumbra_proto::DomainType;

pub mod stateless {
    use super::*;

    pub fn proposal_submit(
        ProposalSubmit {
            proposal,
            deposit_amount: _, // we don't check the deposit amount because it's defined by state
        }: &ProposalSubmit,
    ) -> Result<()> {
        let Proposal {
            id: _, // we can't check the ID statelessly because it's defined by state
            title,
            description: _, // the description can be anything
            payload,
        } = proposal;

        // This is enough room to print "Proposal #999,999: $TITLE" in 99 characters (and the
        // proposal title itself in 80), a decent line width for a modern terminal, as well as a
        // reasonable length for other interfaces
        const PROPOSAL_TITLE_LIMIT: usize = 80;

        if title.len() > PROPOSAL_TITLE_LIMIT {
            return Err(anyhow::anyhow!(
                "proposal title must fit within {PROPOSAL_TITLE_LIMIT} characters"
            ));
        }

        use penumbra_transaction::action::ProposalPayload::*;
        match payload {
            Signaling { commit: _ } => { /* all signaling proposals are valid */ }
            Emergency { halt_chain: _ } => { /* all emergency proposals are valid */ }
            ParameterChange {
                effective_height: _,
                new_parameters,
            } => {
                // Check that new parameters are marked as mutable and within valid bounds
                if !chain_params::is_valid_stateless(new_parameters) {
                    return Err(anyhow::anyhow!("invalid chain parameters"));
                }
            }
            DaoSpend {
                schedule_transactions: _,
                cancel_transactions: _,
            } => {
                // TODO: check that scheduled transactions are valid without any witness or auth data
                anyhow::bail!("DAO spend proposals are not yet supported")
            }
        }

        Ok(())
    }

    pub fn proposal_withdraw(proposal_withdraw: &ProposalWithdraw) -> Result<()> {
        // Enforce a maximum length on proposal withdrawal reasons; 80 characters seems reasonable.
        const PROPOSAL_WITHDRAWAL_REASON_LIMIT: usize = 80;

        if proposal_withdraw.reason.len() > PROPOSAL_WITHDRAWAL_REASON_LIMIT {
            return Err(anyhow::anyhow!(
                "proposal withdrawal reason must fit within {PROPOSAL_WITHDRAWAL_REASON_LIMIT} characters"
            ));
        }

        Ok(())
    }

    pub fn validator_vote(ValidatorVote { body, auth_sig }: &ValidatorVote) -> Result<()> {
        // Check the signature using the GOVERNANCE KEY:
        let body_bytes = body.encode_to_vec();
        body.governance_key
            .0
            .verify(&body_bytes, auth_sig)
            .context("validator vote signature failed to verify")?;

        // This is stateless verification, so we still need to check that the proposal being voted
        // on exists, and that this validator hasn't voted on it already.

        Ok(())
    }

    pub fn delegator_vote(
        DelegatorVote {
            auth_sig,
            proof,
            body:
                DelegatorVoteBody {
                    start_position,
                    value,
                    nullifier,
                    rk,
                    // Unused in stateless checks:
                    vote: _,            // Only used when executing the vote
                    proposal: _,        // Checked against the current open proposals statefully
                    unbonded_amount: _, // Checked against the proposal's snapshot exchange rate statefully
                },
        }: &DelegatorVote,
        context: Arc<Transaction>,
    ) -> Result<()> {
        let effect_hash = context.transaction_body().effect_hash();
        let anchor = context.anchor;

        // 1. Check spend auth signature using provided spend auth key.
        rk.verify(effect_hash.as_ref(), auth_sig)
            .context("delegator vote auth signature failed to verify")?;

        // 2. Check that the proof verifies.

        // Verify the proof with this position as the start position:
        proof
            .verify(anchor, *start_position, *value, *nullifier, *rk)
            .context("a delegator vote proof did not verify")?;

        Ok(())
    }

    pub fn proposal_deposit_claim(
        ProposalDepositClaim {
            // None of these fields can be meaningfully checked without reference to the state
            proposal: _,
            deposit_amount: _,
            outcome: _,
        }: &ProposalDepositClaim,
    ) -> Result<()> {
        // All the checks for this are stateful.
        Ok(())
    }
}

pub mod stateful {
    use super::super::StateReadExt as _;
    use super::*;
    use crate::{
        shielded_pool::{StateReadExt, SupplyRead},
        stake::StateReadExt as _,
    };
    use penumbra_chain::{Epoch, StateReadExt as _};
    use penumbra_crypto::{
        stake::{DelegationToken, IdentityKey},
        Amount, GovernanceKey, Nullifier, Value, STAKING_TOKEN_DENOM,
    };
    use penumbra_storage::StateRead;
    use penumbra_transaction::action::{ProposalDepositClaim, ProposalPayload};

    pub async fn proposal_submit<S: StateRead>(
        state: S,
        ProposalSubmit {
            deposit_amount,
            proposal, // statelessly verified
        }: &ProposalSubmit,
    ) -> Result<()> {
        // Check that the deposit amount agrees with the chain parameters
        let chain_parameters = state.get_chain_params().await?;
        if *deposit_amount != chain_parameters.proposal_deposit_amount {
            anyhow::bail!(
                "submitted proposal deposit of {}{} does not match required proposal deposit of {}{}",
                deposit_amount,
                *STAKING_TOKEN_DENOM,
                chain_parameters.proposal_deposit_amount,
                *STAKING_TOKEN_DENOM,
            );
        }

        // Check that the proposal ID is the correct next proposal ID
        let next_proposal_id = state.next_proposal_id().await?;
        if proposal.id != next_proposal_id {
            anyhow::bail!(
                "submitted proposal ID {} does not match expected proposal ID {}",
                proposal.id,
                next_proposal_id,
            );
        }

        match &proposal.payload {
            ProposalPayload::Signaling { .. } => { /* no stateful checks for signaling */ }
            ProposalPayload::Emergency { .. } => { /* no stateful checks for emergency */ }
            ProposalPayload::ParameterChange {
                effective_height,
                new_parameters,
            } => {
                height_in_future_of_voting_end(&state, *effective_height).await?;

                let old_parameters = state.get_chain_params().await?;

                if !chain_params::is_valid_stateful(new_parameters, &old_parameters) {
                    // TODO: should this return a more descriptive error?
                    return Err(anyhow::anyhow!("invalid chain parameters"));
                }
            }
            ProposalPayload::DaoSpend {
                schedule_transactions,
                cancel_transactions,
            } => {
                for (effective_height, _) in schedule_transactions.iter() {
                    height_in_future_of_voting_end(&state, *effective_height).await?;
                }
                for (scheduled_height, _) in cancel_transactions.iter() {
                    height_in_future_of_voting_end(&state, *scheduled_height).await?;
                }
                // TODO: check that all transactions to cancel exist already and match auth hash
            }
        }

        Ok(())
    }

    async fn height_in_future_of_voting_end<S: StateRead>(state: S, height: u64) -> Result<()> {
        let block_height = state.get_block_height().await?;
        let voting_blocks = state.get_chain_params().await?.proposal_voting_blocks;
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

    pub async fn validator_vote<S: StateRead>(
        state: S,
        ValidatorVote {
            body:
                ValidatorVoteBody {
                    proposal,
                    vote: _, // All votes are valid, so we don't need to do anything with this
                    identity_key,
                    governance_key,
                },
            auth_sig: _, // We already checked this in stateless verification
        }: &ValidatorVote,
    ) -> Result<()> {
        proposal_voteable(&state, *proposal).await?;
        validator_has_not_voted(&state, *proposal, identity_key).await?;
        governance_key_matches_validator(&state, identity_key, governance_key).await?;
        Ok(())
    }

    pub async fn delegator_vote<S: StateRead>(
        state: S,
        DelegatorVote {
            body:
                DelegatorVoteBody {
                    proposal,
                    vote: _, // All votes are valid, so we don't need to do anything with this
                    start_position,
                    value,
                    unbonded_amount,
                    nullifier,
                    rk: _, // We already used this to check the auth sig in stateless verification
                },
            auth_sig: _, // We already checked this in stateless verification
            proof: _,    // We already checked this in stateless verification
        }: &DelegatorVote,
    ) -> Result<()> {
        proposal_voteable(&state, *proposal).await?;
        // TODO: once epochs are non-constant, this has to be re-done to be a stateful lookup:
        let start_block_height = u64::from(
            Epoch::from_height(
                u64::from(start_position.epoch()),
                state.get_chain_params().await?.epoch_duration,
            )
            .start_height(),
        ) + u64::from(start_position.block());
        proposal_started_at_block_height(&state, *proposal, start_block_height).await?;
        nullifier_unspent_before(&state, start_block_height, nullifier).await?;
        nullifier_unvoted_in_proposal(&state, *proposal, nullifier).await?;
        unbonded_amount_correct_exchange(&state, *proposal, value, unbonded_amount).await?;
        Ok(())
    }

    async fn proposal_voteable<S: StateRead>(state: S, proposal_id: u64) -> Result<()> {
        if let Some(proposal_state) = state.proposal_state(proposal_id).await? {
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

    async fn proposal_started_at_block_height<S: StateRead>(
        state: S,
        proposal_id: u64,
        claimed_start_height: u64,
    ) -> Result<()> {
        if let Some(start_height) = state.proposal_voting_start(proposal_id).await? {
            if start_height != claimed_start_height {
                anyhow::bail!(
                    "proposal {} was not started at claimed block height of {}",
                    proposal_id,
                    claimed_start_height
                );
            }
        } else {
            anyhow::bail!("proposal {} does not exist", proposal_id);
        }

        Ok(())
    }

    async fn nullifier_unspent_before<S: StateRead>(
        state: S,
        start_height: u64,
        nullifier: &Nullifier,
    ) -> Result<()> {
        if let Some(spend_info) = state.spend_info(*nullifier).await? {
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

    async fn unbonded_amount_correct_exchange<S: StateRead>(
        state: S,
        proposal_id: u64,
        value: &Value,
        unbonded_amount: &Amount,
    ) -> Result<()> {
        // Attempt to find the denom for the asset ID of the specified value
        let Some(denom) = state.denom_by_asset(&value.asset_id).await? else {
            anyhow::bail!("unknown asset id {} is not a delegation token", value.asset_id);
        };

        // Attempt to find the validator identity for the specified denom, failing if it is not a
        // delegation token
        let validator_identity = DelegationToken::try_from(denom)?.validator();

        // Attempt to look up the snapshotted `RateData` for the validator at the start of the proposal
        let Some(rate_data) = state
            .rate_data_at_proposal_start(proposal_id, validator_identity)
            .await? else {
                anyhow::bail!("validator {} was not active at the start of proposal {}", validator_identity, proposal_id);
            };

        // Check that the unbonded amount is correct relative to that exchange rate
        if rate_data.unbonded_amount(value.amount.into()) != u64::from(*unbonded_amount) {
            anyhow::bail!(
                "unbonded amount {}penumbra does not correspond to {} staked delegation tokens for validator {} using the exchange rate at the start of proposal {}",
                unbonded_amount,
                value.amount,
                validator_identity,
                proposal_id,
            );
        }

        Ok(())
    }

    async fn nullifier_unvoted_in_proposal<S: StateRead>(
        state: S,
        proposal_id: u64,
        nullifier: &Nullifier,
    ) -> Result<()> {
        state
            .per_proposal_check_nullifier_unvoted(proposal_id, nullifier)
            .await
    }

    async fn validator_has_not_voted<S: StateRead>(
        state: S,
        proposal_id: u64,
        identity_key: &IdentityKey,
    ) -> Result<()> {
        if let Some(_vote) = state.validator_vote(proposal_id, *identity_key).await? {
            anyhow::bail!(
                "validator {} has already voted on proposal {}",
                identity_key,
                proposal_id
            );
        }

        Ok(())
    }

    async fn governance_key_matches_validator<S: StateRead>(
        state: S,
        identity_key: &IdentityKey,
        governance_key: &GovernanceKey,
    ) -> Result<()> {
        if let Some(validator) = state.validator(identity_key).await? {
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

    pub async fn proposal_withdraw<S: StateRead>(
        state: S,
        proposal_withdraw: &ProposalWithdraw,
    ) -> Result<()> {
        // Any voteable proposal can be withdrawn
        proposal_voteable(state, proposal_withdraw.proposal).await?;
        Ok(())
    }

    pub async fn proposal_deposit_claim<S: StateRead>(
        state: S,
        proposal_deposit_claim: &ProposalDepositClaim,
    ) -> Result<()> {
        // Any finished proposal can have its deposit claimed
        proposal_claimable(&state, proposal_deposit_claim.proposal).await?;
        // Check that the deposit amount matches the proposal being claimed
        proposal_claim_valid_deposit(
            &state,
            proposal_deposit_claim.proposal,
            proposal_deposit_claim.deposit_amount,
        )
        .await?;
        Ok(())
    }

    async fn proposal_claimable<S: StateRead>(state: S, proposal_id: u64) -> Result<()> {
        if let Some(proposal_state) = state.proposal_state(proposal_id).await? {
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

    async fn proposal_claim_valid_deposit<S: StateRead>(
        state: S,
        proposal_id: u64,
        claim_deposit_amount: Amount,
    ) -> Result<()> {
        if let Some(proposal_deposit_amount) = state.proposal_deposit_amount(proposal_id).await? {
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
