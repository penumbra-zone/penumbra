use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sdk_proto::StateWriteProto as _;
use penumbra_sdk_shielded_pool::component::AssetRegistry;

use crate::{
    action_handler::ActionHandler,
    component::{StateReadExt as _, StateWriteExt},
    event,
    proposal_state::State as ProposalState,
    ProposalNft, ProposalWithdraw,
};

#[async_trait]
impl ActionHandler for ProposalWithdraw {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Enforce a maximum length on proposal withdrawal reasons; 80 characters seems reasonable.
        const PROPOSAL_WITHDRAWAL_REASON_LIMIT: usize = 80;

        if self.reason.len() > PROPOSAL_WITHDRAWAL_REASON_LIMIT {
            anyhow::bail!(
                "proposal withdrawal reason must fit within {PROPOSAL_WITHDRAWAL_REASON_LIMIT} characters"
            );
        }

        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // Any votable proposal can be withdrawn
        state.check_proposal_votable(self.proposal).await?;

        let ProposalWithdraw { proposal, reason } = self;

        // Update the proposal state to withdrawn
        state.put_proposal_state(
            *proposal,
            ProposalState::Withdrawn {
                reason: reason.clone(),
            },
        );

        // Register the denom for the withdrawn proposal NFT
        state
            .register_denom(&ProposalNft::unbonding_deposit(*proposal).denom())
            .await;

        state.record_proto(event::proposal_withdraw(self));

        tracing::debug!(proposal = %proposal, "withdrew proposal");

        Ok(())
    }
}
