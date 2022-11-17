use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
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
            Action::ValidatorDefinition(action) => {
                validator_definition::check_stateless(action, context)
            }
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
            Action::IBCAction(action) => {
                client::Ics2Client::check_stateless(action.clone())?;
                connection::ConnectionComponent::check_stateless(action.clone())?;
                channel::Ics4Channel::check_stateless(action.clone())?;
                Ics20Transfer::check_stateless(action)?;

                Ok(())
            }
            Action::Ics20Withdrawal(action) => action.check_stateless(context),
        }
    }

    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        match self {
            Action::Delegate(action) => action.check_stateful(state, context.clone()).await,
            Action::Undelegate(action) => action.check_stateful(state, context.clone()).await,
            Action::ValidatorDefinition(action) => {
                validator_definition::check_stateful(action, state.clone(), context).await
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
                if tx.ibc_actions().count() > 0 && !state.get_chain_params().await?.ibc_enabled {
                    return Err(anyhow::anyhow!(
                        "transaction contains IBC actions, but IBC is not enabled"
                    ));
                }

                client::Ics2Client::check_tx_stateful(state.clone(), tx.clone()).await?;
                connection::ConnectionComponent::check_tx_stateful(state.clone(), tx.clone())
                    .await?;
                channel::Ics4Channel::check_tx_stateful(state.clone(), tx.clone()).await?;
                Ics20Transfer::check_tx_stateful(state.clone(), tx.clone()).await?;

                Ok(())
            }
            Action::Ics20Withdrawal(action) => action.check_stateful(state, context.clone()).await,
        }
    }

    async fn execute(&self, _state: &mut StateTransaction) -> Result<()> {
        match self {
            Action::Delegate(action) => action.execute(_state).await,
            Action::Undelegate(action) => action.execute(_state).await,
            Action::ValidatorDefinition(_action) => todo!(),
            Action::ValidatorVote(action) => action.execute(_state).await,
            Action::PositionClose(action) => action.execute(_state).await,
            Action::PositionOpen(action) => action.execute(_state).await,
            Action::PositionRewardClaim(action) => action.execute(_state).await,
            Action::PositionWithdraw(action) => action.execute(_state).await,
            Action::ProposalSubmit(action) => action.execute(_state).await,
            Action::ProposalWithdraw(action) => action.execute(_state).await,
            Action::Swap(action) => action.execute(_state).await,
            Action::SwapClaim(action) => action.execute(_state).await,
            Action::Spend(action) => action.execute(_state).await,
            Action::Output(action) => action.execute(_state).await,
            Action::IBCAction(_action) => {
                client::Ics2Client::execute_tx(state, tx.clone()).await?;
                connection::ConnectionComponent::execute_tx(state, tx.clone()).await?;
                channel::Ics4Channel::execute_tx(state, tx.clone()).await?;
                Ics20Transfer::execute_tx(state, tx.clone()).await?;

                Ok(())
            }
            Action::Ics20Withdrawal(action) => action.execute(_state).await,
        }
    }
}
