use std::convert::{TryFrom, TryInto};

use penumbra_proto::{transaction, Protobuf};

// TODO: remove & replace w/ anyhow
pub mod error;

pub mod output;
pub mod spend;

pub use output::Output;
pub use spend::Spend;

/// Supported actions in a Penumbra transaction.
#[derive(Clone, Debug)]
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
            transaction::action::Action::Output(inner) => Ok(Action::Output(inner.try_into()?)),

            transaction::action::Action::Spend(inner) => Ok(Action::Spend(inner.try_into()?)),
        }
    }
}
