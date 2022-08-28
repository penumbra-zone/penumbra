use anyhow::{Context as _, Result};

use super::proposal;
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
            description: _, // the description can be anything
            payload,
        } = proposal;

        use penumbra_transaction::action::ProposalPayload::*;
        match payload {
            Signaling { commit: _ } => { /* all signaling proposals are valid */ }
            Emergency { halt_chain: _ } => {
                anyhow::bail!("emergency proposals are not yet supported")
            }
            ParameterChange {
                effective_height: _,
                new_parameters: _,
            } => anyhow::bail!("parameter change proposals are not yet supported"),
            DaoSpend {
                schedule_transactions: _,
                cancel_transactions: _,
            } => anyhow::bail!("DAO spend proposals are not yet supported"),
        }

        Ok(())
    }

    pub fn proposal_withdraw(_proposal_withdraw: &ProposalWithdraw) -> Result<()> {
        // All the checks are stateful.
        Ok(())
    }

    pub fn validator_vote(ValidatorVote { body, auth_sig }: &ValidatorVote) -> Result<()> {
        // Check the signature:
        let body_bytes = body.encode_to_vec();
        body.identity_key
            .0
            .verify(&body_bytes, auth_sig)
            .context("validator vote signature failed to verify")?;

        // This is stateless verification, so we still need to check that the proposal being voted
        // on exists, and that this validator hasn't voted on it already.

        Ok(())
    }
}

pub mod stateful {
    use super::super::View as _;
    use super::*;
    use penumbra_chain::View as _;
    use penumbra_crypto::{IdentityKey, STAKING_TOKEN_DENOM};
    use penumbra_storage::State;
    use penumbra_transaction::AuthHash;

    pub async fn proposal_submit(
        state: &State,
        ProposalSubmit {
            deposit_amount,
            proposal: _,               // statelessly verified
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
                },
            auth_sig: _, // We already checked this in stateless verification
        }: &ValidatorVote,
    ) -> Result<()> {
        proposal_voteable(state, *proposal).await?;
        validator_has_not_voted(state, *proposal, *identity_key).await?;
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
        identity_key: IdentityKey,
    ) -> Result<()> {
        if let Some(_vote) = state.validator_vote(proposal_id, identity_key).await? {
            anyhow::bail!(
                "validator {} has already voted on proposal {}",
                identity_key,
                proposal_id
            );
        }

        Ok(())
    }
}
