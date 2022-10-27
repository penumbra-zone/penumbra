use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_storage2::State;
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

use crate::governance::view::View as _;
use crate::{Component, Context};

use super::{check, execute, proposal::ProposalList};

pub struct Governance {}

impl Governance {
    pub async fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Component for Governance {
    #[instrument(name = "governance", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {
        // Initialize the unfinished proposals tracking key in the JMT.
        self.state
            .put_unfinished_proposals(ProposalList::default())
            .await;
    }

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

    #[instrument(name = "governance", skip(self, _ctx, _end_block))]
    async fn end_block(&mut self, _ctx: Context, _end_block: &abci::request::EndBlock) {
        // TODO: compute intermediate tallies at epoch boundaries (with threshold delegator voting)
        execute::enact_all_passed_proposals(&self.state).await;
        execute::enact_pending_parameter_changes(&self.state).await;
    }
}
