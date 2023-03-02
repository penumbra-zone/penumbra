use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::StateReadExt as _;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{Action, Transaction};

use super::ActionHandler;

mod dao_deposit;
mod dao_output;
mod dao_spend;
mod delegate;
mod delegator_vote;
mod ibc_action;
mod ics20;
mod output;
mod position;
mod proposal;
mod spend;
mod swap;
mod swap_claim;
mod undelegate;
mod undelegate_claim;
mod validator_definition;
mod validator_vote;

#[async_trait]
impl ActionHandler for Action {
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        match self {
            Action::Delegate(action) => action.check_stateless(context),
            Action::Undelegate(action) => action.check_stateless(context),
            Action::UndelegateClaim(action) => action.check_stateless(context),
            Action::ValidatorDefinition(action) => action.check_stateless(context),
            Action::DelegatorVote(action) => action.check_stateless(context),
            Action::ValidatorVote(action) => action.check_stateless(context),
            Action::PositionClose(action) => action.check_stateless(context),
            Action::PositionOpen(action) => action.check_stateless(context),
            Action::PositionRewardClaim(action) => action.check_stateless(context),
            Action::PositionWithdraw(action) => action.check_stateless(context),
            Action::ProposalSubmit(action) => action.check_stateless(context),
            Action::ProposalWithdraw(action) => action.check_stateless(context),
            Action::ProposalDepositClaim(action) => action.check_stateless(context),
            Action::Swap(action) => action.check_stateless(context),
            Action::SwapClaim(action) => action.check_stateless(context),
            Action::Spend(action) => action.check_stateless(context),
            Action::Output(action) => action.check_stateless(context),
            Action::IBCAction(action) => action.check_stateless(context),
            Action::Ics20Withdrawal(action) => action.check_stateless(context),
            Action::DaoSpend(action) => todo!("check_stateless for dao spend"),
            Action::DaoOutput(action) => todo!("check_stateless for dao output"),
            Action::DaoDeposit(action) => todo!("check_stateless for dao deposit"),
        }
        .await
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        match self {
            Action::Delegate(action) => action.check_stateful(state).await,
            Action::Undelegate(action) => action.check_stateful(state).await,
            Action::UndelegateClaim(action) => action.check_stateful(state).await,
            Action::ValidatorDefinition(action) => action.check_stateful(state).await,
            Action::DelegatorVote(action) => action.check_stateful(state).await,
            Action::ValidatorVote(action) => action.check_stateful(state).await,
            Action::PositionClose(action) => action.check_stateful(state).await,
            Action::PositionOpen(action) => action.check_stateful(state).await,
            Action::PositionRewardClaim(action) => action.check_stateful(state).await,
            Action::PositionWithdraw(action) => action.check_stateful(state).await,
            Action::ProposalSubmit(action) => action.check_stateful(state).await,
            Action::ProposalWithdraw(action) => action.check_stateful(state).await,
            Action::ProposalDepositClaim(action) => action.check_stateful(state).await,
            Action::Swap(action) => action.check_stateful(state).await,
            Action::SwapClaim(action) => action.check_stateful(state).await,
            Action::Spend(action) => action.check_stateful(state).await,
            Action::Output(action) => action.check_stateful(state).await,
            Action::IBCAction(action) => {
                if !state.get_chain_params().await?.ibc_enabled {
                    return Err(anyhow::anyhow!(
                        "transaction contains IBC actions, but IBC is not enabled"
                    ));
                }

                action.check_stateful(state).await
            }
            Action::Ics20Withdrawal(action) => action.check_stateful(state).await,
            Action::DaoSpend(action) => todo!("check_stateful for dao spend"),
            Action::DaoOutput(action) => todo!("check_stateful for dao output"),
            Action::DaoDeposit(action) => todo!("check_stateful for dao deposit"),
        }
    }

    async fn execute<S: StateWrite>(&self, state: S) -> Result<()> {
        match self {
            Action::Delegate(action) => action.execute(state).await,
            Action::Undelegate(action) => action.execute(state).await,
            Action::UndelegateClaim(action) => action.execute(state).await,
            Action::ValidatorDefinition(action) => action.execute(state).await,
            Action::DelegatorVote(action) => action.execute(state).await,
            Action::ValidatorVote(action) => action.execute(state).await,
            Action::PositionClose(action) => action.execute(state).await,
            Action::PositionOpen(action) => action.execute(state).await,
            Action::PositionRewardClaim(action) => action.execute(state).await,
            Action::PositionWithdraw(action) => action.execute(state).await,
            Action::ProposalSubmit(action) => action.execute(state).await,
            Action::ProposalWithdraw(action) => action.execute(state).await,
            Action::ProposalDepositClaim(action) => action.execute(state).await,
            Action::Swap(action) => action.execute(state).await,
            Action::SwapClaim(action) => action.execute(state).await,
            Action::Spend(action) => action.execute(state).await,
            Action::Output(action) => action.execute(state).await,
            Action::IBCAction(action) => action.execute(state).await,
            Action::Ics20Withdrawal(action) => action.execute(state).await,
            Action::DaoSpend(action) => todo!("execute for dao spend"),
            Action::DaoOutput(action) => todo!("execute for dao output"),
            Action::DaoDeposit(action) => todo!("execute for dao deposit"),
        }
    }
}
