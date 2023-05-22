use penumbra_crypto::Balance;
use penumbra_dao::{DaoDeposit, DaoOutput, DaoSpend};
use penumbra_dex::{
    lp::{
        action::{PositionClose, PositionOpen},
        plan::{PositionRewardClaimPlan, PositionWithdrawPlan},
    },
    swap::SwapPlan,
    swap_claim::SwapClaimPlan,
};
use penumbra_ibc::{IbcAction, Ics20Withdrawal};
use penumbra_proto::{core::transaction::v1alpha1 as pb_t, DomainType, TypeUrl};
use penumbra_shielded_pool::{OutputPlan, SpendPlan};
use penumbra_stake::{Delegate, Undelegate, UndelegateClaimPlan};
use serde::{Deserialize, Serialize};

mod delegator_vote;

pub use delegator_vote::DelegatorVotePlan;

use crate::action::{ProposalDepositClaim, ProposalSubmit, ProposalWithdraw, ValidatorVote};

/// A declaration of a planned [`Action`], for use in transaction creation.
///
/// Actions that don't have any private data are passed through, while
/// actions that do are replaced by a "Plan" analogue that includes
/// openings of commitments and other private data.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb_t::ActionPlan", into = "pb_t::ActionPlan")]
#[allow(clippy::large_enum_variant)]
pub enum ActionPlan {
    /// Describes a proposed spend.
    Spend(SpendPlan),
    /// Describes a proposed output.
    Output(OutputPlan),
    /// We don't need any extra information (yet) to understand delegations,
    /// because we don't yet use flow encryption.
    Delegate(Delegate),
    /// We don't need any extra information (yet) to understand undelegations,
    /// because we don't yet use flow encryption.
    Undelegate(Undelegate),
    UndelegateClaim(UndelegateClaimPlan),
    ValidatorDefinition(penumbra_stake::validator::Definition),
    /// Describes a proposed swap.
    Swap(SwapPlan),
    /// Describes a swap claim.
    SwapClaim(SwapClaimPlan),
    IbcAction(IbcAction),
    /// Propose a governance vote.
    ProposalSubmit(ProposalSubmit),
    /// Withdraw a proposed vote.
    ProposalWithdraw(ProposalWithdraw),
    /// Vote on a proposal as a delegator.
    DelegatorVote(DelegatorVotePlan),
    /// Vote on a proposal as a validator.
    ValidatorVote(ValidatorVote),
    /// Claim the deposit for a finished proposal.
    ProposalDepositClaim(ProposalDepositClaim),

    PositionOpen(PositionOpen),
    PositionClose(PositionClose),
    // PositionWithdrawPlan requires the balance of the funds to be withdrawn, so
    // a plan must be used.
    PositionWithdraw(PositionWithdrawPlan),
    // Reward Claim requires the balance of the funds to be claimed, so a plan
    // must be used.
    PositionRewardClaim(PositionRewardClaimPlan),

    DaoSpend(DaoSpend),
    DaoOutput(DaoOutput),
    DaoDeposit(DaoDeposit),

    Withdrawal(Ics20Withdrawal),
}

impl ActionPlan {
    pub fn balance(&self) -> Balance {
        use ActionPlan::*;

        match self {
            Spend(spend) => spend.balance(),
            Output(output) => output.balance(),
            Delegate(delegate) => delegate.balance(),
            Undelegate(undelegate) => undelegate.balance(),
            UndelegateClaim(undelegate_claim) => undelegate_claim.balance(),
            Swap(swap) => swap.balance(),
            SwapClaim(swap_claim) => swap_claim.balance(),
            ProposalSubmit(proposal_submit) => proposal_submit.balance(),
            ProposalWithdraw(proposal_withdraw) => proposal_withdraw.balance(),
            ProposalDepositClaim(proposal_deposit_claim) => proposal_deposit_claim.balance(),
            DelegatorVote(delegator_vote) => delegator_vote.balance(),
            DaoSpend(dao_spend) => dao_spend.balance(),
            DaoOutput(dao_output) => dao_output.balance(),
            DaoDeposit(dao_deposit) => dao_deposit.balance(),
            PositionOpen(position_open) => position_open.balance(),
            PositionClose(position_close) => position_close.balance(),
            PositionWithdraw(position_withdraw) => position_withdraw.balance(),
            PositionRewardClaim(position_reward_claim) => position_reward_claim.balance(),
            Withdrawal(withdrawal) => withdrawal.balance(),
            // None of these contribute to transaction balance:
            IbcAction(_) | ValidatorDefinition(_) | ValidatorVote(_) => Balance::default(),
        }
    }
}

// Convenience impls that make declarative transaction construction easier.

impl From<SpendPlan> for ActionPlan {
    fn from(inner: SpendPlan) -> ActionPlan {
        ActionPlan::Spend(inner)
    }
}

impl From<OutputPlan> for ActionPlan {
    fn from(inner: OutputPlan) -> ActionPlan {
        ActionPlan::Output(inner)
    }
}

impl From<SwapPlan> for ActionPlan {
    fn from(inner: SwapPlan) -> ActionPlan {
        ActionPlan::Swap(inner)
    }
}

impl From<SwapClaimPlan> for ActionPlan {
    fn from(inner: SwapClaimPlan) -> ActionPlan {
        ActionPlan::SwapClaim(inner)
    }
}

impl From<Delegate> for ActionPlan {
    fn from(inner: Delegate) -> ActionPlan {
        ActionPlan::Delegate(inner)
    }
}

impl From<Undelegate> for ActionPlan {
    fn from(inner: Undelegate) -> ActionPlan {
        ActionPlan::Undelegate(inner)
    }
}

impl From<penumbra_stake::validator::Definition> for ActionPlan {
    fn from(inner: penumbra_stake::validator::Definition) -> ActionPlan {
        ActionPlan::ValidatorDefinition(inner)
    }
}

impl From<IbcAction> for ActionPlan {
    fn from(inner: IbcAction) -> ActionPlan {
        ActionPlan::IbcAction(inner)
    }
}

impl From<ProposalSubmit> for ActionPlan {
    fn from(inner: ProposalSubmit) -> ActionPlan {
        ActionPlan::ProposalSubmit(inner)
    }
}

impl From<DelegatorVotePlan> for ActionPlan {
    fn from(inner: DelegatorVotePlan) -> ActionPlan {
        ActionPlan::DelegatorVote(inner)
    }
}

impl From<ValidatorVote> for ActionPlan {
    fn from(inner: ValidatorVote) -> ActionPlan {
        ActionPlan::ValidatorVote(inner)
    }
}

impl From<PositionOpen> for ActionPlan {
    fn from(inner: PositionOpen) -> ActionPlan {
        ActionPlan::PositionOpen(inner)
    }
}

impl From<PositionClose> for ActionPlan {
    fn from(inner: PositionClose) -> ActionPlan {
        ActionPlan::PositionClose(inner)
    }
}

impl From<PositionWithdrawPlan> for ActionPlan {
    fn from(inner: PositionWithdrawPlan) -> ActionPlan {
        ActionPlan::PositionWithdraw(inner)
    }
}

impl From<PositionRewardClaimPlan> for ActionPlan {
    fn from(inner: PositionRewardClaimPlan) -> ActionPlan {
        ActionPlan::PositionRewardClaim(inner)
    }
}

impl From<Ics20Withdrawal> for ActionPlan {
    fn from(inner: Ics20Withdrawal) -> ActionPlan {
        ActionPlan::Withdrawal(inner)
    }
}

impl TypeUrl for ActionPlan {
    const TYPE_URL: &'static str = "/penumbra.core.transaction.v1alpha1.ActionPlan";
}

impl DomainType for ActionPlan {
    type Proto = pb_t::ActionPlan;
}

impl From<ActionPlan> for pb_t::ActionPlan {
    fn from(msg: ActionPlan) -> Self {
        match msg {
            ActionPlan::Output(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::Output(inner.into())),
            },
            ActionPlan::Spend(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::Spend(inner.into())),
            },
            ActionPlan::Delegate(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::Delegate(inner.into())),
            },
            ActionPlan::Undelegate(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::Undelegate(inner.into())),
            },
            ActionPlan::UndelegateClaim(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::UndelegateClaim(inner.into())),
            },
            ActionPlan::ValidatorDefinition(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::ValidatorDefinition(inner.into())),
            },
            ActionPlan::SwapClaim(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::SwapClaim(inner.into())),
            },
            ActionPlan::Swap(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::Swap(inner.into())),
            },
            ActionPlan::IbcAction(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::IbcAction(inner.into())),
            },
            ActionPlan::ProposalSubmit(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::ProposalSubmit(inner.into())),
            },
            ActionPlan::ProposalWithdraw(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::ProposalWithdraw(inner.into())),
            },
            ActionPlan::DelegatorVote(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::DelegatorVote(inner.into())),
            },
            ActionPlan::ValidatorVote(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::ValidatorVote(inner.into())),
            },
            ActionPlan::ProposalDepositClaim(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::ProposalDepositClaim(
                    inner.into(),
                )),
            },
            ActionPlan::PositionOpen(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::PositionOpen(inner.into())),
            },
            ActionPlan::PositionClose(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::PositionClose(inner.into())),
            },
            ActionPlan::PositionWithdraw(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::PositionWithdraw(Into::<
                    penumbra_proto::core::dex::v1alpha1::PositionWithdrawPlan,
                >::into(
                    inner
                ))),
            },
            ActionPlan::PositionRewardClaim(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::PositionRewardClaim(inner.into())),
            },
            ActionPlan::DaoDeposit(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::DaoDeposit(inner.into())),
            },
            ActionPlan::DaoSpend(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::DaoSpend(inner.into())),
            },
            ActionPlan::DaoOutput(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::DaoOutput(inner.into())),
            },
            ActionPlan::Withdrawal(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::Withdrawal(inner.into())),
            },
        }
    }
}

impl TryFrom<pb_t::ActionPlan> for ActionPlan {
    type Error = anyhow::Error;

    fn try_from(proto: pb_t::ActionPlan) -> anyhow::Result<Self, Self::Error> {
        if proto.action.is_none() {
            return Err(anyhow::anyhow!("missing action content"));
        }

        match proto.action.unwrap() {
            pb_t::action_plan::Action::Output(inner) => Ok(ActionPlan::Output(inner.try_into()?)),
            pb_t::action_plan::Action::Spend(inner) => Ok(ActionPlan::Spend(inner.try_into()?)),
            pb_t::action_plan::Action::Delegate(inner) => {
                Ok(ActionPlan::Delegate(inner.try_into()?))
            }
            pb_t::action_plan::Action::Undelegate(inner) => {
                Ok(ActionPlan::Undelegate(inner.try_into()?))
            }
            pb_t::action_plan::Action::UndelegateClaim(inner) => {
                Ok(ActionPlan::UndelegateClaim(inner.try_into()?))
            }
            pb_t::action_plan::Action::ValidatorDefinition(inner) => {
                Ok(ActionPlan::ValidatorDefinition(inner.try_into()?))
            }
            pb_t::action_plan::Action::Swap(inner) => Ok(ActionPlan::Swap(inner.try_into()?)),
            pb_t::action_plan::Action::SwapClaim(inner) => {
                Ok(ActionPlan::SwapClaim(inner.try_into()?))
            }
            pb_t::action_plan::Action::IbcAction(inner) => {
                Ok(ActionPlan::IbcAction(inner.try_into()?))
            }
            pb_t::action_plan::Action::ProposalSubmit(inner) => {
                Ok(ActionPlan::ProposalSubmit(inner.try_into()?))
            }
            pb_t::action_plan::Action::ProposalWithdraw(inner) => {
                Ok(ActionPlan::ProposalWithdraw(inner.try_into()?))
            }
            pb_t::action_plan::Action::ValidatorVote(inner) => {
                Ok(ActionPlan::ValidatorVote(inner.try_into()?))
            }
            pb_t::action_plan::Action::DelegatorVote(inner) => {
                Ok(ActionPlan::DelegatorVote(inner.try_into()?))
            }
            pb_t::action_plan::Action::ProposalDepositClaim(inner) => {
                Ok(ActionPlan::ProposalDepositClaim(inner.try_into()?))
            }
            pb_t::action_plan::Action::PositionOpen(inner) => {
                Ok(ActionPlan::PositionOpen(inner.try_into()?))
            }
            pb_t::action_plan::Action::PositionClose(inner) => {
                Ok(ActionPlan::PositionClose(inner.try_into()?))
            }
            pb_t::action_plan::Action::PositionWithdraw(inner) => {
                Ok(ActionPlan::PositionWithdraw(inner.try_into()?))
            }
            pb_t::action_plan::Action::PositionRewardClaim(inner) => {
                Ok(ActionPlan::PositionRewardClaim(inner.try_into()?))
            }
            pb_t::action_plan::Action::DaoSpend(inner) => {
                Ok(ActionPlan::DaoSpend(inner.try_into()?))
            }
            pb_t::action_plan::Action::DaoDeposit(inner) => {
                Ok(ActionPlan::DaoDeposit(inner.try_into()?))
            }
            pb_t::action_plan::Action::DaoOutput(inner) => {
                Ok(ActionPlan::DaoOutput(inner.try_into()?))
            }
            pb_t::action_plan::Action::Withdrawal(inner) => {
                Ok(ActionPlan::Withdrawal(inner.try_into()?))
            }
        }
    }
}
