use std::convert::{TryFrom, TryInto};

use penumbra_crypto::value;
use penumbra_proto::{ibc as pb_ibc, stake as pbs, transaction as pb, Protobuf};

mod delegate;
pub mod output;
mod propose;
pub mod spend;
mod swap;
mod swap_claim;
mod undelegate;
mod vote;

pub use delegate::Delegate;
pub use output::Output;
pub use propose::{Proposal, ProposalKind, Propose, WithdrawProposal};
pub use spend::Spend;
pub use swap::Swap;
pub use swap_claim::SwapClaim;
pub use undelegate::Undelegate;
pub use vote::{DelegatorVote, ValidatorVote, Vote};

/// An action performed by a Penumbra transaction.
#[derive(Clone, Debug)]
pub enum Action {
    Output(output::Output),
    Spend(spend::Spend),
    Delegate(Delegate),
    Undelegate(Undelegate),
    ValidatorDefinition(pbs::ValidatorDefinition),
    IBCAction(pb_ibc::IbcAction),
    // TODO: re-enable when Swap/SwapClaim is ready
    // Swap(Swap),
    // SwapClaim(SwapClaim),
    Propose(Propose),
    WithdrawProposal(WithdrawProposal),
    DelegatorVote(DelegatorVote),
    ValidatorVote(ValidatorVote),
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
            // TODO: re-enable when Swap/SwapClaim is ready
            // Action::Swap(swap) => swap.value_commitment(),
            // Action::SwapClaim(swap_claim) => swap_claim.value_commitment(),
            // These actions just post data to the chain, and leave the value balance
            // unchanged.
            Action::ValidatorDefinition(_) => value::Commitment::default(),
            Action::IBCAction(_) => value::Commitment::default(),
            Action::Propose(_) => todo!("subtract proposal deposit amount from value balance"),
            Action::WithdrawProposal(_) => value::Commitment::default(),
            Action::DelegatorVote(_) => value::Commitment::default(),
            Action::ValidatorVote(_) => value::Commitment::default(),
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
                action: Some(pb::action::Action::ValidatorDefinition(inner)),
            },
            Action::IBCAction(inner) => pb::Action {
                action: Some(pb::action::Action::IbcAction(inner)),
            },
            Action::Propose(inner) => pb::Action {
                action: Some(pb::action::Action::Propose(inner.into())),
            },
            Action::WithdrawProposal(inner) => pb::Action {
                action: Some(pb::action::Action::WithdrawProposal(inner.into())),
            },
            Action::DelegatorVote(inner) => pb::Action {
                action: Some(pb::action::Action::DelegatorVote(inner.into())),
            },
            Action::ValidatorVote(inner) => pb::Action {
                action: Some(pb::action::Action::ValidatorVote(inner.into())),
            },
            Action::DelegatorVote(inner) => pb::Action {
                action: Some(pb::action::Action::DelegatorVote(inner.into())),
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
                Ok(Action::ValidatorDefinition(inner))
            }
            pb::action::Action::IbcAction(inner) => Ok(Action::IBCAction(inner)),
            pb::action::Action::Propose(inner) => Ok(Action::Propose(inner.try_into()?)),
            pb::action::Action::WithdrawProposal(inner) => {
                Ok(Action::WithdrawProposal(inner.try_into()?))
            }
            pb::action::Action::DelegatorVote(inner) => {
                Ok(Action::DelegatorVote(inner.try_into()?))
            }
            pb::action::Action::ValidatorVote(inner) => {
                Ok(Action::ValidatorVote(inner.try_into()?))
            }
            pb::action::Action::DelegatorVote(inner) => {
                Ok(Action::DelegatorVote(inner.try_into()?))
            }
        }
    }
}
