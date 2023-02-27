use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_crypto::ProposalNft;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::ProposalWithdraw, Transaction};

use crate::{
    action_handler::ActionHandler,
    governance::{proposal, StateReadExt, StateWriteExt},
    shielded_pool::SupplyWrite,
};

#[async_trait]
impl ActionHandler for ProposalWithdraw {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        // Enforce a maximum length on proposal withdrawal reasons; 80 characters seems reasonable.
        const PROPOSAL_WITHDRAWAL_REASON_LIMIT: usize = 80;

        if self.reason.len() > PROPOSAL_WITHDRAWAL_REASON_LIMIT {
            return Err(anyhow::anyhow!(
                "proposal withdrawal reason must fit within {PROPOSAL_WITHDRAWAL_REASON_LIMIT} characters"
            ));
        }

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        // Any votable proposal can be withdrawn
        state.check_proposal_votable(self.proposal).await?;
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let ProposalWithdraw { proposal, reason } = self;

        state
            .put_proposal_state(
                *proposal,
                proposal::State::Withdrawn {
                    reason: reason.clone(),
                },
            )
            .await
            .context("proposal withdraw succeeds")?;

        // Register the denom for the withdrawn proposal NFT
        state
            .register_denom(&ProposalNft::withdrawn(*proposal).denom())
            .await?;

        tracing::debug!(proposal = %proposal, "withdrew proposal");

        Ok(())
    }
}
