use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::Action;

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
    fn check_tx_stateless(&self) -> anyhow::Result<()> {
        match self {
            Action::Delegate(action) => action.check_tx_stateless(),
            Action::Undelegate(action) => action.check_tx_stateless(),
            Action::ValidatorDefinition(_action) => todo!(),
            Action::ValidatorVote(action) => action.check_tx_stateless(),
            Action::PositionClose(action) => action.check_tx_stateless(),
            Action::PositionOpen(action) => action.check_tx_stateless(),
            Action::PositionRewardClaim(action) => action.check_tx_stateless(),
            Action::PositionWithdraw(action) => action.check_tx_stateless(),
            Action::ProposalSubmit(action) => action.check_tx_stateless(),
            Action::ProposalWithdraw(action) => action.check_tx_stateless(),
            Action::Swap(action) => action.check_tx_stateless(),
            Action::SwapClaim(action) => action.check_tx_stateless(),
            Action::Spend(action) => action.check_tx_stateless(),
            Action::Output(action) => action.check_tx_stateless(),
            Action::IBCAction(_action) => todo!(),
            Action::Ics20Withdrawal(action) => action.check_tx_stateless(),
        }
    }

    async fn check_tx_stateful(&self, _state: Arc<State>) -> Result<()> {
        match self {
            Action::Delegate(action) => action.check_tx_stateful(_state).await,
            Action::Undelegate(action) => action.check_tx_stateful(_state).await,
            Action::ValidatorDefinition(_action) => todo!(),
            Action::ValidatorVote(action) => action.check_tx_stateful(_state).await,
            Action::PositionClose(action) => action.check_tx_stateful(_state).await,
            Action::PositionOpen(action) => action.check_tx_stateful(_state).await,
            Action::PositionRewardClaim(action) => action.check_tx_stateful(_state).await,
            Action::PositionWithdraw(action) => action.check_tx_stateful(_state).await,
            Action::ProposalSubmit(action) => action.check_tx_stateful(_state).await,
            Action::ProposalWithdraw(action) => action.check_tx_stateful(_state).await,
            Action::Swap(action) => action.check_tx_stateful(_state).await,
            Action::SwapClaim(action) => action.check_tx_stateful(_state).await,
            Action::Spend(action) => action.check_tx_stateful(_state).await,
            Action::Output(action) => action.check_tx_stateful(_state).await,
            Action::IBCAction(_action) => todo!(),
            Action::Ics20Withdrawal(action) => action.check_tx_stateful(_state).await,
        }
    }

    async fn execute_tx(&self, _state: &mut StateTransaction) -> Result<()> {
        match self {
            Action::Delegate(action) => action.execute_tx(_state).await,
            Action::Undelegate(action) => action.execute_tx(_state).await,
            Action::ValidatorDefinition(_action) => todo!(),
            Action::ValidatorVote(action) => action.execute_tx(_state).await,
            Action::PositionClose(action) => action.execute_tx(_state).await,
            Action::PositionOpen(action) => action.execute_tx(_state).await,
            Action::PositionRewardClaim(action) => action.execute_tx(_state).await,
            Action::PositionWithdraw(action) => action.execute_tx(_state).await,
            Action::ProposalSubmit(action) => action.execute_tx(_state).await,
            Action::ProposalWithdraw(action) => action.execute_tx(_state).await,
            Action::Swap(action) => action.execute_tx(_state).await,
            Action::SwapClaim(action) => action.execute_tx(_state).await,
            Action::Spend(action) => action.execute_tx(_state).await,
            Action::Output(action) => action.execute_tx(_state).await,
            Action::IBCAction(_action) => todo!(),
            Action::Ics20Withdrawal(action) => action.execute_tx(_state).await,
        }
    }
}
