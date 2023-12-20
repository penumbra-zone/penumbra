use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_chain::TransactionContext;
use penumbra_ibc::component::StateReadExt as _;
use penumbra_shielded_pool::component::Ics20Transfer;
use penumbra_transaction::Action;

mod submit;

use crate::state_delta_wrapper::StateDeltaWrapper;

use super::ActionHandler;
use cnidarium_component::ActionHandler as _;

#[async_trait]
impl ActionHandler for Action {
    type CheckStatelessContext = TransactionContext;

    async fn check_stateless(&self, context: TransactionContext) -> Result<()> {
        match self {
            // These actions require a context
            Action::SwapClaim(action) => action.check_stateless(context).await,
            Action::Spend(action) => action.check_stateless(context).await,
            Action::DelegatorVote(action) => action.check_stateless(context).await,
            // These actions don't require a context
            Action::Delegate(action) => action.check_stateless(()).await,
            Action::Undelegate(action) => action.check_stateless(()).await,
            Action::UndelegateClaim(action) => action.check_stateless(()).await,
            Action::ValidatorDefinition(action) => action.check_stateless(()).await,
            Action::ValidatorVote(action) => action.check_stateless(()).await,
            Action::PositionClose(action) => action.check_stateless(()).await,
            Action::PositionOpen(action) => action.check_stateless(()).await,
            Action::PositionRewardClaim(action) => action.check_stateless(()).await,
            Action::PositionWithdraw(action) => action.check_stateless(()).await,
            Action::ProposalSubmit(action) => action.check_stateless(()).await,
            Action::ProposalWithdraw(action) => action.check_stateless(()).await,
            Action::ProposalDepositClaim(action) => action.check_stateless(()).await,
            Action::Swap(action) => action.check_stateless(()).await,
            Action::Output(action) => action.check_stateless(()).await,
            Action::IbcRelay(action) => {
                action
                    .clone()
                    .with_handler::<Ics20Transfer>()
                    .check_stateless(())
                    .await
            }
            Action::Ics20Withdrawal(action) => action.check_stateless(()).await,
            Action::DaoSpend(action) => action.check_stateless(()).await,
            Action::DaoOutput(action) => action.check_stateless(()).await,
            Action::DaoDeposit(action) => action.check_stateless(()).await,
        }
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
            Action::IbcRelay(action) => {
                if !state.get_ibc_params().await?.ibc_enabled {
                    anyhow::bail!("transaction contains IBC actions, but IBC is not enabled");
                }

                action
                    .clone()
                    .with_handler::<Ics20Transfer>()
                    .check_stateful(state)
                    .await
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
            Action::IbcRelay(action) => {
                let mut state = state;
                let wrapper = StateDeltaWrapper(&mut state);
                action
                    .clone()
                    .with_handler::<Ics20Transfer>()
                    .execute(wrapper)
                    .await
            }
            Action::Ics20Withdrawal(action) => action.execute(state).await,
            Action::DaoSpend(action) => action.execute(state).await,
            Action::DaoOutput(action) => action.execute(state).await,
            Action::DaoDeposit(action) => action.execute(state).await,
        }
    }
}
