use anyhow::{Context as _, Result};

use super::proposal;
use penumbra_crypto::rdsa::{SpendAuth, VerificationKey};
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

    pub fn proposal_withdraw(ProposalWithdraw { body, auth_sig }: &ProposalWithdraw) -> Result<()> {
        // Check the signature:
        let body_bytes = body.encode_to_vec();
        body.withdraw_proposal_key
            .verify(&body_bytes, auth_sig)
            .context("proposal withdraw signature failed to verify")?;

        // This is stateless verification, so we still need to check that the verification key
        // provided is the *same* as the one submitted when the proposal was created, that the
        // proposal exists in the first place, and that it is either pre-voting or during the voting
        // period.

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
    use penumbra_crypto::IdentityKey;
    use penumbra_storage::State;

    pub async fn proposal_submit(_state: &State, _proposal_submit: &ProposalSubmit) -> Result<()> {
        // No stateful checks necessary for proposal submit
        Ok(())
    }

    pub async fn proposal_withdraw(
        state: &State,
        ProposalWithdraw {
            body:
                ProposalWithdrawBody {
                    proposal,
                    withdraw_proposal_key,
                },
            auth_sig: _, // We already checked this in stateless verification
        }: &ProposalWithdraw,
    ) -> Result<()> {
        proposal_withdrawable(state, *proposal).await?;
        proposal_withdraw_key_matches(state, *proposal, withdraw_proposal_key).await?;
        Ok(())
    }

    async fn proposal_withdrawable(state: &State, proposal_id: u64) -> Result<()> {
        if let Some(proposal_state) = state.proposal_state(proposal_id).await? {
            use proposal::State::*;
            match proposal_state {
                Proposed | Voting => {
                    // You can withdraw a proposal that is either proposed or in voting period presently
                }
                Withdrawn => anyhow::bail!("proposal {} has already been withdrawn", proposal_id),
                Finished { .. } => {
                    anyhow::bail!("voting on proposal {} has already concluded", proposal_id)
                }
            }
        } else {
            anyhow::bail!("proposal {} does not exist", proposal_id);
        }

        Ok(())
    }

    async fn proposal_withdraw_key_matches(
        state: &State,
        proposal_id: u64,
        withdraw_proposal_key: &VerificationKey<SpendAuth>,
    ) -> Result<()> {
        if let Some(stored_key) = state.proposal_withdraw_key(proposal_id).await? {
            if stored_key != *withdraw_proposal_key {
                anyhow::bail!(
                    "withdrawal key for proposal {} does not match signature on withdrawal",
                    proposal_id
                );
            }
        } else {
            anyhow::bail!("proposal {} does not exist", proposal_id);
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
                Proposed => anyhow::bail!(
                    "proposal {} has not yet started its voting period",
                    proposal_id
                ),
                Voting => {
                    // This is when you can vote on a proposal
                }
                Withdrawn => anyhow::bail!("proposal {} has already been withdrawn", proposal_id),
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
