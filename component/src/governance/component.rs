use anyhow::{Context as _, Result};
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_crypto::{
    rdsa::{SpendAuth, VerificationKey},
    IdentityKey,
};
use penumbra_storage::{State, StateExt};
use penumbra_transaction::{action::Vote, Transaction};
use tendermint::abci;
use tracing::instrument;

use crate::{Component, Context};

use super::{check, event, execute, metrics, proposal, state_key};

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
        for proposal_submit in tx.proposal_submits() {
            check::stateful::proposal_submit(&self.state, proposal_submit).await?;
        }
        for proposal_withdraw in tx.proposal_withdraws() {
            check::stateful::proposal_withdraw(&self.state, proposal_withdraw).await?;
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

    #[instrument(name = "governance", skip(self, ctx, tx))]
    async fn execute_tx(&mut self, ctx: Context, tx: &Transaction) {
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
        // TODO: if epoch boundary, activate proposals that are entering voting, and conclude
        // proposals that are exiting voting
    }
}

impl Governance {
    pub async fn new(state: State) -> Self {
        Self { state }
    }
}

#[async_trait]
pub trait View: StateExt {
    /// Get the id of the next proposal in the sequence of ids.
    async fn next_proposal_id(&self) -> Result<u64> {
        Ok(self
            .get_proto::<u64>(state_key::latest_proposal_id().into())
            .await?
            .map(|i| i + 1)
            .unwrap_or(0))
    }

    /// Get a fresh, unique, incrementing proposal id.
    async fn fresh_proposal_id(&self) -> Result<u64> {
        let next_proposal_id = self.next_proposal_id().await?;

        // Record this proposal id, so we won't re-use it
        self.put_proto(state_key::latest_proposal_id().into(), next_proposal_id)
            .await;

        // Return the new proposal id
        Ok(next_proposal_id)
    }

    /// Get the state of a proposal.
    async fn proposal_state(&self, proposal_id: u64) -> Result<Option<proposal::State>> {
        Ok(self
            .get_domain::<proposal::State, _>(state_key::proposal_state(proposal_id).into())
            .await?)
    }

    /// Get the withdrawal key of a proposal.
    async fn proposal_withdraw_key(
        &self,
        proposal_id: u64,
    ) -> Result<Option<VerificationKey<SpendAuth>>> {
        Ok(self
            .get_domain::<VerificationKey<SpendAuth>, _>(
                state_key::proposal_withdraw_key(proposal_id).into(),
            )
            .await?)
    }

    /// Get the vote of a validator on a particular proposal.
    async fn validator_vote(
        &self,
        proposal_id: u64,
        identity_key: IdentityKey,
    ) -> Result<Option<Vote>> {
        Ok(self
            .get_domain::<Vote, _>(state_key::validator_vote(proposal_id, identity_key).into())
            .await?)
    }
}

impl<T: StateExt> View for T {}
