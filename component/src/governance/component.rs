use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_storage2::{State, StateTransaction};
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

use super::view::StateWriteExt as _;
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
    //#[instrument(name = "governance", skip(state, _app_state))]
    async fn init_chain(state: &mut StateTransaction, _app_state: &genesis::AppState) {
        // Initialize the unfinished proposals tracking key in the JMT.
        state
            .put_unfinished_proposals(ProposalList::default())
            .await;
    }

    #[instrument(name = "governance", skip(_state, _ctx, _begin_block))]
    async fn begin_block(
        _state: &mut StateTransaction,
        _ctx: Context,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "governance", skip(_ctx, tx))]
    fn check_tx_stateless(_ctx: Context, tx: Arc<Transaction>) -> Result<()> {
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

    #[instrument(name = "governance", skip(state, _ctx, tx))]
    async fn check_tx_stateful(
        state: Arc<State>,
        _ctx: Context,
        tx: Arc<Transaction>,
    ) -> Result<()> {
        let auth_hash = tx.transaction_body().auth_hash();

        for proposal_submit in tx.proposal_submits() {
            check::stateful::proposal_submit(&state, proposal_submit).await?;
        }
        for proposal_withdraw in tx.proposal_withdraws() {
            check::stateful::proposal_withdraw(&state, &auth_hash, proposal_withdraw).await?;
        }
        for validator_vote in tx.validator_votes() {
            check::stateful::validator_vote(&state, validator_vote).await?;
        }
        // TODO: fill in when delegator votes happen
        // for delegator_vote in tx.delegator_votes() {
        //     check::stateful::delegator_vote(&self.state, delegator_vote).await?;
        // }

        Ok(())
    }

    #[instrument(name = "governance", skip(state, _ctx, tx))]
    async fn execute_tx(
        state: &mut StateTransaction,
        _ctx: Context,
        tx: Arc<Transaction>,
    ) -> Result<()> {
        for proposal_submit in tx.proposal_submits() {
            execute::proposal_submit(state, proposal_submit).await;
        }
        for proposal_withdraw in tx.proposal_withdraws() {
            execute::proposal_withdraw(state, proposal_withdraw).await;
        }
        for validator_vote in tx.validator_votes() {
            execute::validator_vote(state, validator_vote).await;
        }
        // TODO: fill in when delegator votes happen
        // for delegator_vote in tx.delegator_votes() {
        //     execute::delegator_vote(&self.state, delegator_vote).await;
        // }
        Ok(())
    }

    #[instrument(name = "governance", skip(state, _ctx, _end_block))]
    async fn end_block(
        state: &mut StateTransaction,
        _ctx: Context,
        _end_block: &abci::request::EndBlock,
    ) {
        // TODO: compute intermediate tallies at epoch boundaries (with threshold delegator voting)
        execute::enact_all_passed_proposals(state).await;
        execute::enact_pending_parameter_changes(state).await;
    }
}
