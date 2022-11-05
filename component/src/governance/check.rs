use anyhow::{Context as _, Result};

use super::proposal::{self, chain_params};
use penumbra_transaction::action::{
    ProposalSubmit, ProposalWithdraw, ProposalWithdrawBody, ValidatorVote, ValidatorVoteBody,
};

pub mod stateless {
    use penumbra_proto::Protobuf;
    use penumbra_transaction::action::Proposal;

    use super::*;

    pub fn proposal_submit(
        ProposalSubmit {
            proposal,
            deposit_refund_address: _, // the refund address can be any valid address
            deposit_amount: _, // we don't check the deposit amount because it's defined by state
            withdraw_proposal_key: _, // the withdraw proposal key can be any valid key
        }: &ProposalSubmit,
    ) -> Result<()> {
        let Proposal {
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

    pub fn proposal_withdraw(_proposal_withdraw: &ProposalWithdraw) -> Result<()> {
        // All the checks are stateful.
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
}

pub mod stateful {

    use super::super::StateReadExt as _;
    use super::*;
    use crate::stake::StateReadExt as _;
    use penumbra_chain::StateReadExt as _;
    use penumbra_crypto::{GovernanceKey, IdentityKey, STAKING_TOKEN_DENOM};
    use penumbra_storage2::State;
    use penumbra_transaction::{action::ProposalPayload, AuthHash};

    pub async fn proposal_submit(
        state: &State,
        ProposalSubmit {
            deposit_amount,
            proposal,                  // statelessly verified
            deposit_refund_address: _, // can be anything
            withdraw_proposal_key: _,  // can be any valid key
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

        match &proposal.payload {
            ProposalPayload::Signaling { .. } => { /* no stateful checks for signaling */ }
            ProposalPayload::Emergency { .. } => { /* no stateful checks for emergency */ }
            ProposalPayload::ParameterChange {
                effective_height,
                new_parameters,
            } => {
                height_in_future_of_voting_end(state, *effective_height).await?;

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
                    height_in_future_of_voting_end(state, *effective_height).await?;
                }
                for (scheduled_height, _) in cancel_transactions.iter() {
                    height_in_future_of_voting_end(state, *scheduled_height).await?;
                }
                // TODO: check that all transactions to cancel exist already and match auth hash
            }
        }

        Ok(())
    }

    async fn height_in_future_of_voting_end(state: &State, height: u64) -> Result<()> {
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

    pub async fn proposal_withdraw(
        state: &State,
        auth_hash: &AuthHash,
        proposal_withdraw @ ProposalWithdraw {
            body:
                ProposalWithdrawBody {
                    proposal,
                    reason: _, // Nothing is done here
                },
            auth_sig: _, // We already checked this in stateless verification
        }: &ProposalWithdraw,
    ) -> Result<()> {
        proposal_withdrawable(state, *proposal).await?;
        proposal_withdraw_key_verifies(state, auth_hash, proposal_withdraw).await?;
        Ok(())
    }

    async fn proposal_withdrawable(state: &State, proposal_id: u64) -> Result<()> {
        if let Some(proposal_state) = state.proposal_state(proposal_id).await? {
            use proposal::State::*;
            match proposal_state {
                Voting => {
                    // You can withdraw a proposal that is currently voting
                }
                Withdrawn { .. } => {
                    anyhow::bail!("proposal {} has already been withdrawn", proposal_id)
                }
                Finished { .. } => {
                    anyhow::bail!("voting on proposal {} has already concluded", proposal_id)
                }
            }
        } else {
            anyhow::bail!("proposal {} does not exist", proposal_id);
        }

        Ok(())
    }

    async fn proposal_withdraw_key_verifies(
        state: &State,
        auth_hash: &AuthHash,
        ProposalWithdraw { body, auth_sig }: &ProposalWithdraw,
    ) -> Result<()> {
        if let Some(withdraw_proposal_key) = state.proposal_withdrawal_key(body.proposal).await? {
            withdraw_proposal_key
                .verify(auth_hash.as_ref(), auth_sig)
                .context("proposal withdraw signature failed to verify")?;
        } else {
            anyhow::bail!("proposal {} does not exist", body.proposal);
        }

        Ok(())
    }

    pub async fn validator_vote(
        state: &State,
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
        proposal_voteable(state, *proposal).await?;
        validator_has_not_voted(state, *proposal, identity_key).await?;
        governance_key_matches_validator(state, identity_key, governance_key).await?;
        Ok(())
    }

    async fn proposal_voteable(state: &State, proposal_id: u64) -> Result<()> {
        if let Some(proposal_state) = state.proposal_state(proposal_id).await? {
            use proposal::State::*;
            match proposal_state {
                Voting => {
                    // This is when you can vote on a proposal
                }
                Withdrawn { .. } => {
                    anyhow::bail!("proposal {} has already been withdrawn", proposal_id)
                }
                Finished { .. } => {
                    anyhow::bail!("voting on proposal {} has already concluded", proposal_id)
                }
            }
        } else {
            anyhow::bail!("proposal {} does not exist", proposal_id);
        }

        Ok(())
    }

    async fn validator_has_not_voted(
        state: &State,
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

    async fn governance_key_matches_validator(
        state: &State,
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
}
