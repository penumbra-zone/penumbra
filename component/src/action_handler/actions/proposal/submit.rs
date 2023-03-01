use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_chain::StateReadExt as _;
use penumbra_crypto::{ProposalNft, VotingReceiptToken, STAKING_TOKEN_DENOM};
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::action::proposal::{PROPOSAL_DESCRIPTION_LIMIT, PROPOSAL_TITLE_LIMIT};
use penumbra_transaction::action::{proposal, Proposal, ProposalPayload};
use penumbra_transaction::{action::ProposalSubmit, Transaction};

use crate::action_handler::ActionHandler;
use crate::governance::{StateReadExt as _, StateWriteExt as _};
use crate::shielded_pool::{StateReadExt, StateWriteExt as _, SupplyWrite};

#[async_trait]
impl ActionHandler for ProposalSubmit {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        let ProposalSubmit {
            proposal,
            deposit_amount: _, // we don't check the deposit amount because it's defined by state
        } = self;
        let Proposal {
            id: _, // we can't check the ID statelessly because it's defined by state
            title,
            description,
            payload,
        } = proposal;

        if title.len() > PROPOSAL_TITLE_LIMIT {
            return Err(anyhow::anyhow!(
                "proposal title must fit within {PROPOSAL_TITLE_LIMIT} characters"
            ));
        }

        if description.len() > PROPOSAL_DESCRIPTION_LIMIT {
            return Err(anyhow::anyhow!(
                "proposal description must fit within {PROPOSAL_DESCRIPTION_LIMIT} characters"
            ));
        }

        use penumbra_transaction::action::ProposalPayload::*;
        match payload {
            Signaling { commit: _ } => { /* all signaling proposals are valid */ }
            Emergency { halt_chain: _ } => { /* all emergency proposals are valid */ }
            ParameterChange { old, new } => {
                old.check_valid_update(new)
                    .context("invalid change to chain parameters")?;
            }
            DaoSpend {
                transaction_plan: _,
            } => {
                // TODO: check that scheduled transactions are valid without any witness or auth data
                anyhow::bail!("DAO spend proposals are not yet supported")
            }
        }

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        let ProposalSubmit {
            deposit_amount,
            proposal, // statelessly verified
        } = self;

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
            ProposalPayload::ParameterChange { .. } => {
                /* no stateful checks for parameter change (checks are applied when proposal finishes) */
            }
            ProposalPayload::DaoSpend { .. } => {
                /* no stateful checks for DAO spend (checks are applied when proposal finishes) */
            }
        }

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let ProposalSubmit {
            proposal,
            deposit_amount,
        } = self;

        // Store the contents of the proposal and generate a fresh proposal id for it
        let proposal_id = state
            .new_proposal(proposal)
            .await
            .context("can create proposal")?;

        // Set the deposit amount for the proposal
        state.put_deposit_amount(proposal_id, *deposit_amount);

        // Register the denom for the voting proposal NFT
        state
            .register_denom(&ProposalNft::voting(proposal_id).denom())
            .await?;

        // Register the denom for the vote receipt tokens
        state
            .register_denom(&VotingReceiptToken::new(proposal_id).denom())
            .await?;

        // Set the proposal state to voting (votes start immediately)
        state.put_proposal_state(proposal_id, proposal::State::Voting);

        // Determine what block it is currently, and calculate when the proposal should start voting
        // (now!) and finish voting (later...), then write that into the state
        let chain_params = state
            .get_chain_params()
            .await
            .context("can get chain params")?;
        let current_block = state
            .get_block_height()
            .await
            .context("can get block height")?;
        let voting_end = current_block + chain_params.proposal_voting_blocks;
        state.put_proposal_voting_start(proposal_id, current_block);
        state.put_proposal_voting_end(proposal_id, voting_end);

        // Compute the effective starting TCT position for the proposal, by rounding the current
        // position down to the start of the block.
        let Some(sct_position) = state.stub_state_commitment_tree().await.position() else {
            anyhow::bail!("state commitment tree is full");
        };
        // All proposals start are considered to start at the beginning of the block, because this
        // means there are no ordering games to be played within the block in which a proposal begins:
        let proposal_start_position = (sct_position.epoch(), sct_position.block(), 0).into();
        state.put_proposal_voting_start_position(proposal_id, proposal_start_position);

        // If there was a proposal submitted, ensure we track this so that clients
        // can retain state needed to vote as delegators
        let mut compact_block = state.stub_compact_block();
        compact_block.proposal_started = true;
        state.stub_put_compact_block(compact_block);

        tracing::debug!(proposal = %proposal_id, "created proposal");

        Ok(())
    }
}
