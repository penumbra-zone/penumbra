use penumbra_proto::{ibc as pb_ibc, stake as pb_stake, transaction as pb_t, Protobuf};
use serde::{Deserialize, Serialize};

mod output;
mod spend;

pub use output::OutputPlan;
pub use spend::SpendPlan;

use crate::action::{Delegate, Undelegate};

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
        }
    }
}
