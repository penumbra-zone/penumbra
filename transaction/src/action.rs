use std::convert::{TryFrom, TryInto};

use penumbra_crypto::value;
use penumbra_ibc as ibc;
use penumbra_proto::{transaction as pb, Protobuf};
use penumbra_stake as stake;

pub mod output;
pub mod spend;

pub use output::Output;
pub use spend::Spend;

/// Supported actions in a Penumbra transaction.
#[derive(Clone, Debug)]
pub enum Action {
    Output(output::Output),
    Spend(spend::Spend),
    Delegate(stake::Delegate),
    Undelegate(stake::Undelegate),
    ValidatorDefinition(stake::ValidatorDefinition),
    IBCAction(ibc::IBCAction),
}

impl Action {
    /// Obtains or computes a commitment to the (typed) value added or subtracted from
    /// the transaction's balance by this action.
    pub fn value_commitment(&self) -> value::Commitment {
        match self {
            Action::Output(output) => output.body.value_commitment,
            Action::Spend(spend) => spend.body.value_commitment,
            Action::Delegate(delegate) => delegate.value_commitment(),
            Action::Undelegate(undelegate) => undelegate.value_commitment(),
            Action::ValidatorDefinition(_) => value::Commitment::default(),
            // TODO: should IBC actions have value commitments?
            Action::IBCAction(_) => value::Commitment::default(),
        }
    }
}

impl Protobuf<pb::Action> for Action {}

impl From<Action> for pb::Action {
    fn from(msg: Action) -> Self {
        match msg {
            Action::Output(inner) => pb::Action {
                action: Some(pb::action::Action::Output(inner.into())),
            },
            Action::Spend(inner) => pb::Action {
                action: Some(pb::action::Action::Spend(inner.into())),
            },
            Action::Delegate(inner) => pb::Action {
                action: Some(pb::action::Action::Delegate(inner.into())),
            },
            Action::Undelegate(inner) => pb::Action {
                action: Some(pb::action::Action::Undelegate(inner.into())),
            },
            Action::ValidatorDefinition(inner) => pb::Action {
                action: Some(pb::action::Action::ValidatorDefinition(inner.into())),
            },
            Action::IBCAction(inner) => pb::Action {
                action: Some(pb::action::Action::IbcAction(inner.into())),
            },
        }
    }
}

impl TryFrom<pb::Action> for Action {
    type Error = anyhow::Error;

    fn try_from(proto: pb::Action) -> anyhow::Result<Self, Self::Error> {
        if proto.action.is_none() {
            return Err(anyhow::anyhow!("missing action content"));
        }

        match proto.action.unwrap() {
            pb::action::Action::Output(inner) => Ok(Action::Output(inner.try_into()?)),

            pb::action::Action::Spend(inner) => Ok(Action::Spend(inner.try_into()?)),
            pb::action::Action::Delegate(inner) => Ok(Action::Delegate(inner.try_into()?)),
            pb::action::Action::Undelegate(inner) => Ok(Action::Undelegate(inner.try_into()?)),
            pb::action::Action::ValidatorDefinition(inner) => {
                Ok(Action::ValidatorDefinition(inner.try_into()?))
            }
            pb::action::Action::IbcAction(inner) => Ok(Action::IBCAction(inner.try_into()?)),
        }
    }
}
