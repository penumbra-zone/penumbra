use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::StateReadExt as _;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{Action, Transaction};

use super::ActionHandler;

mod delegate;
mod ibc_action;
mod ics20;
mod output;
mod position;
mod proposal;
mod spend;
mod swap;
mod swap_claim;
mod undelegate;
mod validator_definition;
mod validator_vote;

#[async_trait]
impl ActionHandler for Action {
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        match self {
            Action::Delegate(action) => action.check_stateless(context),
            Action::Undelegate(action) => action.check_stateless(context),
            Action::ValidatorDefinition(action) => action.check_stateless(context),
            Action::ValidatorVote(action) => action.check_stateless(context),
            Action::PositionClose(action) => action.check_stateless(context),
            Action::PositionOpen(action) => action.check_stateless(context),
            Action::PositionRewardClaim(action) => action.check_stateless(context),
            Action::PositionWithdraw(action) => action.check_stateless(context),
            Action::ProposalSubmit(action) => action.check_stateless(context),
            Action::ProposalWithdraw(action) => action.check_stateless(context),
            Action::Swap(action) => action.check_stateless(context),
            Action::SwapClaim(action) => action.check_stateless(context),
            Action::Spend(action) => action.check_stateless(context),
            Action::Output(action) => action.check_stateless(context),
            Action::IBCAction(action) => action.check_stateless(context),
            Action::Ics20Withdrawal(action) => action.check_stateless(context),
        }
    }

    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        match self {
            Action::Delegate(action) => action.check_stateful(state, context.clone()).await,
            Action::Undelegate(action) => action.check_stateful(state, context.clone()).await,
            Action::ValidatorDefinition(action) => {
                action.check_stateful(state, context.clone()).await
            }
            Action::ValidatorVote(action) => action.check_stateful(state, context.clone()).await,
            Action::PositionClose(action) => action.check_stateful(state, context.clone()).await,
            Action::PositionOpen(action) => action.check_stateful(state, context.clone()).await,
            Action::PositionRewardClaim(action) => {
                action.check_stateful(state, context.clone()).await
            }
            Action::PositionWithdraw(action) => action.check_stateful(state, context.clone()).await,
            Action::ProposalSubmit(action) => action.check_stateful(state, context.clone()).await,
            Action::ProposalWithdraw(action) => action.check_stateful(state, context.clone()).await,
            Action::Swap(action) => action.check_stateful(state, context.clone()).await,
            Action::SwapClaim(action) => action.check_stateful(state, context.clone()).await,
            Action::Spend(action) => action.check_stateful(state, context.clone()).await,
            Action::Output(action) => action.check_stateful(state, context.clone()).await,
            Action::IBCAction(action) => {
                if context.ibc_actions().count() > 0 && !state.get_chain_params().await?.ibc_enabled
                {
                    return Err(anyhow::anyhow!(
                        "transaction contains IBC actions, but IBC is not enabled"
                    ));
                }

                action.check_stateful(state, context.clone()).await
            }
            Action::Ics20Withdrawal(action) => action.check_stateful(state, context.clone()).await,
        }
    }

    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        match self {
            Action::Delegate(action) => action.execute(state).await,
            Action::Undelegate(action) => action.execute(state).await,
            Action::ValidatorDefinition(action) => action.execute(state).await,
            Action::ValidatorVote(action) => action.execute(state).await,
            Action::PositionClose(action) => action.execute(state).await,
            Action::PositionOpen(action) => action.execute(state).await,
            Action::PositionRewardClaim(action) => action.execute(state).await,
            Action::PositionWithdraw(action) => action.execute(state).await,
            Action::ProposalSubmit(action) => action.execute(state).await,
            Action::ProposalWithdraw(action) => action.execute(state).await,
            Action::Swap(action) => action.execute(state).await,
            Action::SwapClaim(action) => action.execute(state).await,
            Action::Spend(action) => action.execute(state).await,
            Action::Output(action) => action.execute(state).await,
            Action::IBCAction(action) => action.execute(state).await,
            Action::Ics20Withdrawal(action) => action.execute(state).await,
        }
    }
}
