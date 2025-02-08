use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_shielded_pool::component::Ics20Transfer;
use penumbra_sdk_transaction::Action;
use penumbra_sdk_txhash::TransactionContext;

mod submit;

use crate::PenumbraHost;

use super::AppActionHandler;
use cnidarium_component::ActionHandler as _;

#[async_trait]
impl AppActionHandler for Action {
    type CheckStatelessContext = TransactionContext;

    async fn check_stateless(&self, context: TransactionContext) -> Result<()> {
        match self {
            Action::SwapClaim(action) => action.check_stateless(context).await,
            Action::Spend(action) => action.check_stateless(context).await,
            Action::DelegatorVote(action) => action.check_stateless(context).await,
            Action::Delegate(action) => action.check_stateless(()).await,
            Action::Undelegate(action) => action.check_stateless(()).await,
            Action::UndelegateClaim(action) => action.check_stateless(()).await,
            Action::ValidatorDefinition(action) => action.check_stateless(()).await,
            Action::ValidatorVote(action) => action.check_stateless(()).await,
            Action::PositionClose(action) => action.check_stateless(()).await,
            Action::PositionOpen(action) => action.check_stateless(()).await,
            Action::PositionWithdraw(action) => action.check_stateless(()).await,
            Action::ProposalSubmit(action) => action.check_stateless(()).await,
            Action::ProposalWithdraw(action) => action.check_stateless(()).await,
            Action::ProposalDepositClaim(action) => action.check_stateless(()).await,
            Action::Swap(action) => action.check_stateless(()).await,
            Action::Output(action) => action.check_stateless(()).await,
            Action::IbcRelay(action) => {
                action
                    .clone()
                    .with_handler::<Ics20Transfer, PenumbraHost>()
                    .check_stateless(())
                    .await
            }
            Action::Ics20Withdrawal(action) => {
                action
                    .clone()
                    .with_handler::<PenumbraHost>()
                    .check_stateless(())
                    .await
            }
            Action::CommunityPoolSpend(action) => action.check_stateless(()).await,
            Action::CommunityPoolOutput(action) => action.check_stateless(()).await,
            Action::CommunityPoolDeposit(action) => action.check_stateless(()).await,
            Action::ActionDutchAuctionSchedule(action) => action.check_stateless(()).await,
            Action::ActionDutchAuctionEnd(action) => action.check_stateless(()).await,
            Action::ActionDutchAuctionWithdraw(action) => action.check_stateless(()).await,
            Action::ActionLiquidityTournamentVote(action) => action.check_stateless(context).await,
        }
    }

    async fn check_historical<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        match self {
            Action::Delegate(action) => action.check_historical(state).await,
            Action::Undelegate(action) => action.check_historical(state).await,
            Action::UndelegateClaim(action) => action.check_historical(state).await,
            Action::ValidatorDefinition(action) => action.check_historical(state).await,
            Action::DelegatorVote(action) => action.check_historical(state).await,
            Action::ValidatorVote(action) => action.check_historical(state).await,
            Action::PositionClose(action) => action.check_historical(state).await,
            Action::PositionOpen(action) => action.check_historical(state).await,
            Action::PositionWithdraw(action) => action.check_historical(state).await,
            Action::ProposalSubmit(action) => action.check_historical(state).await,
            Action::ProposalWithdraw(action) => action.check_historical(state).await,
            Action::ProposalDepositClaim(action) => action.check_historical(state).await,
            Action::Swap(action) => action.check_historical(state).await,
            Action::SwapClaim(action) => action.check_historical(state).await,
            Action::Spend(action) => action.check_historical(state).await,
            Action::Output(action) => action.check_historical(state).await,
            Action::IbcRelay(action) => {
                action
                    .clone()
                    .with_handler::<Ics20Transfer, PenumbraHost>()
                    .check_historical(state)
                    .await
            }
            Action::Ics20Withdrawal(action) => {
                action
                    .clone()
                    .with_handler::<PenumbraHost>()
                    .check_historical(state)
                    .await
            }
            Action::CommunityPoolSpend(action) => action.check_historical(state).await,
            Action::CommunityPoolOutput(action) => action.check_historical(state).await,
            Action::CommunityPoolDeposit(action) => action.check_historical(state).await,
            Action::ActionDutchAuctionSchedule(action) => action.check_historical(state).await,
            Action::ActionDutchAuctionEnd(action) => action.check_historical(state).await,
            Action::ActionDutchAuctionWithdraw(action) => action.check_historical(state).await,
            Action::ActionLiquidityTournamentVote(action) => action.check_historical(state).await,
        }
    }

    async fn check_and_execute<S: StateWrite>(&self, state: S) -> Result<()> {
        match self {
            Action::Delegate(action) => action.check_and_execute(state).await,
            Action::Undelegate(action) => action.check_and_execute(state).await,
            Action::UndelegateClaim(action) => action.check_and_execute(state).await,
            Action::ValidatorDefinition(action) => action.check_and_execute(state).await,
            Action::DelegatorVote(action) => action.check_and_execute(state).await,
            Action::ValidatorVote(action) => action.check_and_execute(state).await,
            Action::PositionClose(action) => action.check_and_execute(state).await,
            Action::PositionOpen(action) => action.check_and_execute(state).await,
            Action::PositionWithdraw(action) => action.check_and_execute(state).await,
            Action::ProposalSubmit(action) => action.check_and_execute(state).await,
            Action::ProposalWithdraw(action) => action.check_and_execute(state).await,
            Action::ProposalDepositClaim(action) => action.check_and_execute(state).await,
            Action::Swap(action) => action.check_and_execute(state).await,
            Action::SwapClaim(action) => action.check_and_execute(state).await,
            Action::Spend(action) => action.check_and_execute(state).await,
            Action::Output(action) => action.check_and_execute(state).await,
            Action::IbcRelay(action) => {
                action
                    .clone()
                    .with_handler::<Ics20Transfer, PenumbraHost>()
                    .check_and_execute(state)
                    .await
            }
            Action::Ics20Withdrawal(action) => {
                action
                    .clone()
                    .with_handler::<PenumbraHost>()
                    .check_and_execute(state)
                    .await
            }
            Action::CommunityPoolSpend(action) => action.check_and_execute(state).await,
            Action::CommunityPoolOutput(action) => action.check_and_execute(state).await,
            Action::CommunityPoolDeposit(action) => action.check_and_execute(state).await,
            Action::ActionDutchAuctionSchedule(action) => action.check_and_execute(state).await,
            Action::ActionDutchAuctionEnd(action) => action.check_and_execute(state).await,
            Action::ActionDutchAuctionWithdraw(action) => action.check_and_execute(state).await,
            Action::ActionLiquidityTournamentVote(action) => action.check_and_execute(state).await,
        }
    }
}
