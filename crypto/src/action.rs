use penumbra_proto::{transaction, Protobuf};
use std::convert::{TryFrom, TryInto};

pub mod constants;
pub mod error;
pub mod output;
pub mod spend;

/// Supported actions in a Penumbra transaction.
pub enum Action {
    Output(output::Output),
    Spend(spend::Spend),
}

impl Protobuf<transaction::Action> for Action {}

impl From<Action> for transaction::Action {
    fn from(msg: Action) -> Self {
        match msg {
            Action::Output(inner) => transaction::Action {
                action: Some(transaction::action::Action::Output(inner.into())),
            },
            Action::Spend(inner) => transaction::Action {
                action: Some(transaction::action::Action::Spend(inner.into())),
            },
        }
    }
}

impl TryFrom<transaction::Action> for Action {
    type Error = error::ProtoError;

    fn try_from(proto: transaction::Action) -> anyhow::Result<Self, Self::Error> {
        if proto.action.is_none() {
            return Err(error::ProtoError::ActionMalformed);
        }

        match proto.action.unwrap() {
            transaction::action::Action::Output(inner) => {
                return Ok(Action::Output(inner.try_into()?));
            }

            transaction::action::Action::Spend(inner) => {
                return Ok(Action::Spend(inner.try_into()?));
            }
        }
    }
}
