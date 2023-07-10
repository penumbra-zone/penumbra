use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::component::StateReadExt as _;
use penumbra_chain::TransactionContext;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Action;

use super::ActionHandler;
use penumbra_component::ActionHandler as _;

mod delegator_vote;
mod proposal;
mod validator_vote;

#[async_trait]
impl ActionHandler for Action {
    type CheckStatelessContext = TransactionContext;

    async fn check_stateless(&self, context: TransactionContext) -> Result<()> {
        match self {
            // These actions require a context
            Action::SwapClaim(action) => action.check_stateless(context),
            Action::Spend(action) => action.check_stateless(context),
            Action::DelegatorVote(action) => action.check_stateless(context),
            // These actions don't require a context
            Action::Delegate(action) => action.check_stateless(()),
            Action::Undelegate(action) => action.check_stateless(()),
            Action::UndelegateClaim(action) => action.check_stateless(()),
            Action::ValidatorDefinition(action) => action.check_stateless(()),
            Action::ValidatorVote(action) => action.check_stateless(()),
            Action::PositionClose(action) => action.check_stateless(()),
            Action::PositionOpen(action) => action.check_stateless(()),
            Action::PositionRewardClaim(action) => action.check_stateless(()),
            Action::PositionWithdraw(action) => action.check_stateless(()),
            Action::ProposalSubmit(action) => action.check_stateless(()),
            Action::ProposalWithdraw(action) => action.check_stateless(()),
            Action::ProposalDepositClaim(action) => action.check_stateless(()),
            Action::Swap(action) => action.check_stateless(()),
            Action::Output(action) => action.check_stateless(()),
            Action::IbcAction(action) => action.check_stateless(()),
            Action::Ics20Withdrawal(action) => action.check_stateless(()),
            Action::DaoSpend(action) => action.check_stateless(()),
            Action::DaoOutput(action) => action.check_stateless(()),
            Action::DaoDeposit(action) => action.check_stateless(()),
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
            Action::IbcAction(action) => {
                if !state.get_chain_params().await?.ibc_enabled {
                    return Err(anyhow::anyhow!(
                        "transaction contains IBC actions, but IBC is not enabled"
                    ));
                }

                action.check_stateful(state).await
            }
            Action::Ics20Withdrawal(action) => action.check_stateful(state).await,
            Action::DaoSpend(action) => action.check_stateful(state).await,
            Action::DaoOutput(action) => action.check_stateful(state).await,
            Action::DaoDeposit(action) => action.check_stateful(state).await,
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
            Action::IbcAction(action) => action.execute(state).await,
            Action::Ics20Withdrawal(action) => action.execute(state).await,
            Action::DaoSpend(action) => action.execute(state).await,
            Action::DaoOutput(action) => action.execute(state).await,
            Action::DaoDeposit(action) => action.execute(state).await,
        }
    }
}
