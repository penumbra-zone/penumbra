use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_crypto::ProposalNft;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::action::proposal::Outcome;
use penumbra_transaction::{action::ProposalDepositClaim, Transaction};

use crate::action_handler::ActionHandler;
use crate::governance::{proposal, StateReadExt as _, StateWriteExt as _};
use crate::shielded_pool::SupplyWrite;

#[async_trait]
impl ActionHandler for ProposalDepositClaim {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        // No stateless checks are required for this action (all checks require state access)
        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        // Any finished proposal can have its deposit claimed
        state.check_proposal_claimable(self.proposal).await?;
        // Check that the deposit amount matches the proposal being claimed
        state
            .check_proposal_claim_valid_deposit(self.proposal, self.deposit_amount)
            .await?;
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let ProposalDepositClaim {
            proposal,
            deposit_amount: _, // not needed to transition state; deposit is self-minted in tx
            outcome: resupplied_outcome,
        } = self;

        // The only effect of doing a deposit claim is to state transition the proposal to claimed so it
        // cannot be claimed again. The deposit amount is self-minted in the transaction (proof of
        // deserving-ness is the supplied proposal NFT, which is burned in the transaction), so we don't
        // need to distribute it here.

        if let Some(proposal::State::Finished { outcome }) = state.proposal_state(*proposal).await?
        {
            // This should be prevented by earlier checks, but replicating here JUST IN CASE!
            if *resupplied_outcome != outcome.as_ref().map(|_| ()) {
                anyhow::bail!(
                    "proposal {} has outcome {:?}, but deposit claim has outcome {:?}",
                    proposal,
                    outcome,
                    resupplied_outcome
                );
            }

            // Register the denom for the claimed proposal NFT
            state
                .register_denom(
                    &match &outcome {
                        Outcome::Passed => ProposalNft::passed(*proposal),
                        Outcome::Failed { .. } => ProposalNft::failed(*proposal),
                        Outcome::Vetoed { .. } => ProposalNft::vetoed(*proposal),
                    }
                    .denom(),
                )
                .await?;

            // Set the proposal state to claimed
            state.put_proposal_state(*proposal, proposal::State::Claimed { outcome });
        } else {
            anyhow::bail!("proposal {} is not in finished state", proposal);
        }

        Ok(())
    }
}
