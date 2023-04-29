use penumbra_ibc::{IbcAction, Ics20Withdrawal};
use penumbra_proto::core::stake::v1alpha1::ValidatorDefinition;
use penumbra_proto::{core::transaction::v1alpha1 as pbt, DomainType};
use serde::{Deserialize, Serialize};

pub mod delegator_vote_view;
pub mod swap_claim_view;
pub mod swap_view;

pub use delegator_vote_view::DelegatorVoteView;
pub use penumbra_shielded_pool::OutputView;
pub use penumbra_shielded_pool::SpendView;
pub use swap_claim_view::SwapClaimView;
pub use swap_view::SwapView;

use crate::action::{
    DaoDeposit, DaoOutput, DaoSpend, Delegate, PositionClose, PositionOpen, PositionRewardClaim,
    PositionWithdraw, ProposalDepositClaim, ProposalSubmit, ProposalWithdraw, Undelegate,
    UndelegateClaim, ValidatorVote,
};
use crate::Action;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::ActionView", into = "pbt::ActionView")]
#[allow(clippy::large_enum_variant)]
pub enum ActionView {
    // Action types with encrypted contents
    Spend(SpendView),
    Output(OutputView),
    Swap(SwapView),
    SwapClaim(SwapClaimView),
    DelegatorVote(DelegatorVoteView),
    // Action types with transparent contents
    ValidatorDefinition(ValidatorDefinition),
    IbcAction(IbcAction),
    ProposalSubmit(ProposalSubmit),
    ProposalWithdraw(ProposalWithdraw),
    ValidatorVote(ValidatorVote),
    ProposalDepositClaim(ProposalDepositClaim),
    PositionOpen(PositionOpen),
    PositionClose(PositionClose),
    PositionWithdraw(PositionWithdraw),
    PositionRewardClaim(PositionRewardClaim),
    Delegate(Delegate),
    Undelegate(Undelegate),
    UndelegateClaim(UndelegateClaim),
    Ics20Withdrawal(Ics20Withdrawal),
    DaoDeposit(DaoDeposit),
    DaoSpend(DaoSpend),
    DaoOutput(DaoOutput),
}

impl DomainType for ActionView {
    type Proto = pbt::ActionView;
}

impl TryFrom<pbt::ActionView> for ActionView {
    type Error = anyhow::Error;

    fn try_from(v: pbt::ActionView) -> Result<Self, Self::Error> {
        use pbt::action_view::ActionView as AV;
        Ok(
            match v
                .action_view
                .ok_or_else(|| anyhow::anyhow!("missing action_view"))?
            {
                AV::Delegate(x) => ActionView::Delegate(x.try_into()?),
                AV::Spend(x) => ActionView::Spend(x.try_into()?),
                AV::Output(x) => ActionView::Output(x.try_into()?),
                AV::Undelegate(x) => ActionView::Undelegate(x.try_into()?),
                AV::UndelegateClaim(x) => ActionView::UndelegateClaim(x.try_into()?),
                AV::Swap(x) => ActionView::Swap(x.try_into()?),
                AV::SwapClaim(x) => ActionView::SwapClaim(x.try_into()?),
                AV::ValidatorDefinition(x) => ActionView::ValidatorDefinition(x),
                AV::IbcAction(x) => ActionView::IbcAction(x.try_into()?),
                AV::ProposalSubmit(x) => ActionView::ProposalSubmit(x.try_into()?),
                AV::ProposalWithdraw(x) => ActionView::ProposalWithdraw(x.try_into()?),
                AV::ProposalDepositClaim(x) => ActionView::ProposalDepositClaim(x.try_into()?),
                AV::ValidatorVote(x) => ActionView::ValidatorVote(x.try_into()?),
                AV::DelegatorVote(x) => ActionView::DelegatorVote(x.try_into()?),
                AV::PositionOpen(x) => ActionView::PositionOpen(x.try_into()?),
                AV::PositionClose(x) => ActionView::PositionClose(x.try_into()?),
                AV::PositionWithdraw(x) => ActionView::PositionWithdraw(x.try_into()?),
                AV::PositionRewardClaim(x) => ActionView::PositionRewardClaim(x.try_into()?),
                AV::Ics20Withdrawal(x) => ActionView::Ics20Withdrawal(x.try_into()?),
                AV::DaoDeposit(x) => ActionView::DaoDeposit(x.try_into()?),
                AV::DaoSpend(x) => ActionView::DaoSpend(x.try_into()?),
                AV::DaoOutput(x) => ActionView::DaoOutput(x.try_into()?),
            },
        )
    }
}

impl From<ActionView> for pbt::ActionView {
    fn from(v: ActionView) -> Self {
        use pbt::action_view::ActionView as AV;
        Self {
            action_view: Some(match v {
                ActionView::Swap(x) => AV::Swap(x.into()),
                ActionView::SwapClaim(x) => AV::SwapClaim(x.into()),
                ActionView::Output(x) => AV::Output(x.into()),
                ActionView::Spend(x) => AV::Spend(x.into()),
                ActionView::Delegate(x) => AV::Delegate(x.into()),
                ActionView::Undelegate(x) => AV::Undelegate(x.into()),
                ActionView::UndelegateClaim(x) => AV::UndelegateClaim(x.into()),
                // FIXME: this shouldn't be using a proto type internally
                ActionView::ValidatorDefinition(x) => AV::ValidatorDefinition(x),
                ActionView::IbcAction(x) => AV::IbcAction(x.into()),
                ActionView::ProposalSubmit(x) => AV::ProposalSubmit(x.into()),
                ActionView::ProposalWithdraw(x) => AV::ProposalWithdraw(x.into()),
                ActionView::ValidatorVote(x) => AV::ValidatorVote(x.into()),
                ActionView::DelegatorVote(x) => AV::DelegatorVote(x.into()),
                ActionView::ProposalDepositClaim(x) => AV::ProposalDepositClaim(x.into()),
                ActionView::PositionOpen(x) => AV::PositionOpen(x.into()),
                ActionView::PositionClose(x) => AV::PositionClose(x.into()),
                ActionView::PositionWithdraw(x) => AV::PositionWithdraw(x.into()),
                ActionView::PositionRewardClaim(x) => AV::PositionRewardClaim(x.into()),
                ActionView::Ics20Withdrawal(x) => AV::Ics20Withdrawal(x.into()),
                ActionView::DaoDeposit(x) => AV::DaoDeposit(x.into()),
                ActionView::DaoSpend(x) => AV::DaoSpend(x.into()),
                ActionView::DaoOutput(x) => AV::DaoOutput(x.into()),
            }),
        }
    }
}

impl From<ActionView> for Action {
    fn from(action_view: ActionView) -> Action {
        match action_view {
            ActionView::Swap(x) => Action::Swap(x.into()),
            ActionView::SwapClaim(x) => Action::SwapClaim(x.into()),
            ActionView::Output(x) => Action::Output(x.into()),
            ActionView::Spend(x) => Action::Spend(x.into()),
            ActionView::Delegate(x) => Action::Delegate(x),
            ActionView::Undelegate(x) => Action::Undelegate(x),
            ActionView::UndelegateClaim(x) => Action::UndelegateClaim(x),
            ActionView::ValidatorDefinition(x) => Action::ValidatorDefinition(x),
            ActionView::IbcAction(x) => Action::IbcAction(x),
            ActionView::ProposalSubmit(x) => Action::ProposalSubmit(x),
            ActionView::ProposalWithdraw(x) => Action::ProposalWithdraw(x),
            ActionView::ValidatorVote(x) => Action::ValidatorVote(x),
            ActionView::DelegatorVote(x) => Action::DelegatorVote(x.into()),
            ActionView::ProposalDepositClaim(x) => Action::ProposalDepositClaim(x),
            ActionView::PositionOpen(x) => Action::PositionOpen(x),
            ActionView::PositionClose(x) => Action::PositionClose(x),
            ActionView::PositionWithdraw(x) => Action::PositionWithdraw(x),
            ActionView::PositionRewardClaim(x) => Action::PositionRewardClaim(x),
            ActionView::Ics20Withdrawal(x) => Action::Ics20Withdrawal(x),
            ActionView::DaoDeposit(x) => Action::DaoDeposit(x),
            ActionView::DaoSpend(x) => Action::DaoSpend(x),
            ActionView::DaoOutput(x) => Action::DaoOutput(x),
        }
    }
}
