use penumbra_proto::{ibc as pb_ibc, stake as pb_stake, transaction as pb_t, Protobuf};
use serde::{Deserialize, Serialize};

mod delegator_vote;
mod output;
mod spend;

pub use delegator_vote::DelegatorVotePlan;
pub use output::OutputPlan;
pub use spend::SpendPlan;

use crate::action::{
    Delegate, ProposalSubmit, ProposalWithdrawBody, Undelegate, ValidatorVoteBody,
};

/// A declaration of a planned [`Action`], for use in transaction creation.
///
/// Actions that don't have any private data are passed through, while
/// actions that do are replaced by a "Plan" analogue that includes
/// openings of commitments and other private data.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb_t::ActionPlan", into = "pb_t::ActionPlan")]
pub enum ActionPlan {
    /// Describes a proposed spend.
    Spend(SpendPlan),
    /// We don't need any extra information to understand well-formed outputs, since we can decrypt with the OVK.
    Output(OutputPlan),
    /// We don't need any extra information (yet) to understand delegations,
    /// because we don't yet use flow encryption.
    Delegate(Delegate),
    /// We don't need any extra information (yet) to understand undelegations,
    /// because we don't yet use flow encryption.
    Undelegate(Undelegate),
    ValidatorDefinition(pb_stake::ValidatorDefinition),
    IBCAction(pb_ibc::IbcAction),
    /// Propose a governance vote.
    ProposalSubmit(ProposalSubmit),
    /// Withdraw a proposed vote.
    ProposalWithdraw(ProposalWithdrawBody),
    /// Vote on a proposal as a delegator.
    DelegatorVote(DelegatorVotePlan),
    /// Vote on a proposal as a validator.
    ValidatorVote(ValidatorVoteBody),
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

impl From<pb_stake::ValidatorDefinition> for ActionPlan {
    fn from(inner: pb_stake::ValidatorDefinition) -> ActionPlan {
        ActionPlan::ValidatorDefinition(inner)
    }
}

impl From<pb_ibc::IbcAction> for ActionPlan {
    fn from(inner: pb_ibc::IbcAction) -> ActionPlan {
        ActionPlan::IBCAction(inner)
    }
}

impl From<ProposalSubmit> for ActionPlan {
    fn from(inner: ProposalSubmit) -> ActionPlan {
        ActionPlan::ProposalSubmit(inner)
    }
}

impl From<ProposalWithdrawBody> for ActionPlan {
    fn from(inner: ProposalWithdrawBody) -> ActionPlan {
        ActionPlan::ProposalWithdraw(inner)
    }
}

impl From<DelegatorVotePlan> for ActionPlan {
    fn from(inner: DelegatorVotePlan) -> ActionPlan {
        ActionPlan::DelegatorVote(inner)
    }
}

impl From<ValidatorVoteBody> for ActionPlan {
    fn from(inner: ValidatorVoteBody) -> ActionPlan {
        ActionPlan::ValidatorVote(inner)
    }
}

impl Protobuf<pb_t::ActionPlan> for ActionPlan {}

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
            ActionPlan::ValidatorDefinition(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::ValidatorDefinition(inner)),
            },
            ActionPlan::IBCAction(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::IbcAction(inner)),
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
            pb_t::action_plan::Action::ValidatorDefinition(inner) => {
                Ok(ActionPlan::ValidatorDefinition(inner))
            }
            pb_t::action_plan::Action::IbcAction(inner) => Ok(ActionPlan::IBCAction(inner)),
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
        }
    }
}
