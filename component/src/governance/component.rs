use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_storage::State;
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

use crate::{Component, Context};

use super::{check, execute, proposal, tally, View as _};

pub struct Governance {
    state: State,
}

#[async_trait]
impl Component for Governance {
    #[instrument(name = "governance", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {}

    #[instrument(name = "governance", skip(self, _ctx, _begin_block))]
    async fn begin_block(&mut self, _ctx: Context, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "governance", skip(_ctx, tx))]
    fn check_tx_stateless(_ctx: Context, tx: &Transaction) -> Result<()> {
        for proposal_submit in tx.proposal_submits() {
            check::stateless::proposal_submit(proposal_submit)?;
        }
        for proposal_withdraw in tx.proposal_withdraws() {
            check::stateless::proposal_withdraw(proposal_withdraw)?;
        }
        for validator_vote in tx.validator_votes() {
            check::stateless::validator_vote(validator_vote)?;
        }
        // TODO: fill in when delegator votes happen
        // for delegator_vote in tx.delegator_votes() {
        //     check::stateless::delegator_vote(delegator_vote)?;
        // }

        Ok(())
    }

    #[instrument(name = "governance", skip(self, _ctx, tx))]
    async fn check_tx_stateful(&self, _ctx: Context, tx: &Transaction) -> Result<()> {
        let auth_hash = tx.transaction_body().auth_hash();

        for proposal_submit in tx.proposal_submits() {
            check::stateful::proposal_submit(&self.state, proposal_submit).await?;
        }
        for proposal_withdraw in tx.proposal_withdraws() {
            check::stateful::proposal_withdraw(&self.state, &auth_hash, proposal_withdraw).await?;
        }
        for validator_vote in tx.validator_votes() {
            check::stateful::validator_vote(&self.state, validator_vote).await?;
        }
        // TODO: fill in when delegator votes happen
        // for delegator_vote in tx.delegator_votes() {
        //     check::stateful::delegator_vote(&self.state, delegator_vote).await?;
        // }

        Ok(())
    }

    #[instrument(name = "governance", skip(self, _ctx, tx))]
    async fn execute_tx(&mut self, _ctx: Context, tx: &Transaction) {
        for proposal_submit in tx.proposal_submits() {
            execute::proposal_submit(&self.state, proposal_submit).await;
        }
        for proposal_withdraw in tx.proposal_withdraws() {
            execute::proposal_withdraw(&self.state, proposal_withdraw).await;
        }
        for validator_vote in tx.validator_votes() {
            execute::validator_vote(&self.state, validator_vote).await;
        }
        // TODO: fill in when delegator votes happen
        // for delegator_vote in tx.delegator_votes() {
        //     execute::delegator_vote(&self.state, delegator_vote).await;
        // }
    }

    #[instrument(name = "governance", skip(self, _ctx, end_block))]
    async fn end_block(&mut self, _ctx: Context, end_block: &abci::request::EndBlock) {
        let parameters = tally::Parameters::new(&self.state)
            .await
            .expect("can generate tally parameters");

        let height = end_block.height as u64;

        let circumstance = tally::Circumstance::new(&self.state)
            .await
            .expect("can generate tally circumstance");

        // For every unfinished proposal, conclude those that finish in this block
        for proposal_id in self
            .state
            .unfinished_proposals()
            .await
            .expect("can get unfinished proposals")
        {
            // TODO: tally delegator votes
            if let Some(outcome) = parameters
                .tally(&self.state, circumstance, proposal_id)
                .await
                .expect("can tally proposal")
            {
                tracing::debug!(proposal = %proposal_id, outcome = ?outcome, "proposal voting finished");

                // If the outcome was not vetoed, issue a refund of the proposal deposit --
                // otherwise, the deposit will never be refunded, and therefore is burned
                if outcome.should_be_refunded() {
                    self.state
                        .add_proposal_refund(height, proposal_id)
                        .await
                        .expect("can add proposal refund");
                }

                tracing::debug!(proposal = %proposal_id, "issuing proposal deposit refund");

                // Record the outcome of the proposal
                self.state
                    .put_proposal_state(proposal_id, proposal::State::Finished { outcome })
                    .await
                    .expect("can put finished proposal outcome");
            } else {
                tracing::debug!(proposal = %proposal_id, "burning proposal deposit for vetoed proposal");
            }
        }

        // TODO: Compute intermediate tallies at epoch boundaries (with threshold delegator voting)
    }
}

impl Governance {
    pub async fn new(state: State) -> Self {
        Self { state }
    }
}
