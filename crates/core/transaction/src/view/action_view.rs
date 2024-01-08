use penumbra_community_pool::{CommunityPoolDeposit, CommunityPoolOutput, CommunityPoolSpend};
use penumbra_dex::{
    lp::action::{PositionClose, PositionOpen, PositionRewardClaim, PositionWithdraw},
    swap::SwapView,
    swap_claim::SwapClaimView,
};
use penumbra_governance::{ProposalDepositClaim, ProposalSubmit, ProposalWithdraw, ValidatorVote};
use penumbra_ibc::IbcRelay;
use penumbra_proto::{core::transaction::v1alpha1 as pbt, DomainType};
use penumbra_shielded_pool::Ics20Withdrawal;
use penumbra_stake::{Delegate, Undelegate, UndelegateClaim};
use serde::{Deserialize, Serialize};

pub use penumbra_governance::DelegatorVoteView;
pub use penumbra_shielded_pool::OutputView;
pub use penumbra_shielded_pool::SpendView;

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
    ValidatorDefinition(penumbra_stake::validator::Definition),
    IbcRelay(IbcRelay),
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
    CommunityPoolDeposit(CommunityPoolDeposit),
    CommunityPoolSpend(CommunityPoolSpend),
    CommunityPoolOutput(CommunityPoolOutput),
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
                AV::ValidatorDefinition(x) => ActionView::ValidatorDefinition(x.try_into()?),
                AV::IbcRelayAction(x) => ActionView::IbcRelay(x.try_into()?),
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
                AV::CommunityPoolDeposit(x) => ActionView::CommunityPoolDeposit(x.try_into()?),
                AV::CommunityPoolSpend(x) => ActionView::CommunityPoolSpend(x.try_into()?),
                AV::CommunityPoolOutput(x) => ActionView::CommunityPoolOutput(x.try_into()?),
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
                ActionView::ValidatorDefinition(x) => AV::ValidatorDefinition(x.into()),
                ActionView::IbcRelay(x) => AV::IbcRelayAction(x.into()),
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
                ActionView::CommunityPoolDeposit(x) => AV::CommunityPoolDeposit(x.into()),
                ActionView::CommunityPoolSpend(x) => AV::CommunityPoolSpend(x.into()),
                ActionView::CommunityPoolOutput(x) => AV::CommunityPoolOutput(x.into()),
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
            ActionView::IbcRelay(x) => Action::IbcRelay(x),
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
            ActionView::CommunityPoolDeposit(x) => Action::CommunityPoolDeposit(x),
            ActionView::CommunityPoolSpend(x) => Action::CommunityPoolSpend(x),
            ActionView::CommunityPoolOutput(x) => Action::CommunityPoolOutput(x),
        }
    }
}
