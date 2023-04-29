use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_crypto::ProposalNft;
use penumbra_shielded_pool::component::SupplyWrite;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::ProposalWithdraw, proposal};

use crate::{
    action_handler::ActionHandler,
    governance::{StateReadExt, StateWriteExt},
};

#[async_trait]
impl ActionHandler for ProposalWithdraw {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
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

        // Update the proposal state to withdrawn
        state.put_proposal_state(
            *proposal,
            proposal::State::Withdrawn {
                reason: reason.clone(),
            },
        );

        // Register the denom for the withdrawn proposal NFT
        state
            .register_denom(&ProposalNft::unbonding_deposit(*proposal).denom())
            .await?;

        tracing::debug!(proposal = %proposal, "withdrew proposal");

        Ok(())
    }
}
