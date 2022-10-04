pub mod output_view;
pub mod spend_view;
pub mod swap_claim_view;
pub mod swap_view;

pub use output_view::OutputView;
use penumbra_proto::core::{ibc::v1alpha1::IbcAction, stake::v1alpha1::ValidatorDefinition};
pub use spend_view::SpendView;
pub use swap_claim_view::SwapClaimView;
pub use swap_view::SwapView;

use crate::action::{
    Delegate, ICS20Withdrawal, PositionClose, PositionOpen, PositionRewardClaim, PositionWithdraw,
    ProposalSubmit, ProposalWithdraw, Undelegate, ValidatorVote,
};
#[allow(clippy::large_enum_variant)]
pub enum ActionView {
    // Action types with encrypted contents
    Swap(SwapView),
    SwapClaim(SwapClaimView),
    Output(OutputView),
    Spend(SpendView),
    // Action types with transparent contents
    Delegate(Delegate),
    Undelegate(Undelegate),
    ValidatorDefinition(ValidatorDefinition),
    IBCAction(IbcAction),
    ProposalSubmit(ProposalSubmit),
    ProposalWithdraw(ProposalWithdraw),
    ValidatorVote(ValidatorVote),
    PositionOpen(PositionOpen),
    PositionClose(PositionClose),
    PositionWithdraw(PositionWithdraw),
    PositionRewardClaim(PositionRewardClaim),
    ICS20Withdrawal(ICS20Withdrawal),
}
