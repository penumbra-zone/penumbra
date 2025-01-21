use anyhow::anyhow;
use penumbra_sdk_auction::auction::dutch::actions::{
    ActionDutchAuctionEnd, ActionDutchAuctionSchedule, ActionDutchAuctionWithdraw,
};
use penumbra_sdk_txhash::{EffectHash, EffectingData};
use std::convert::{TryFrom, TryInto};

use penumbra_sdk_asset::balance;
use penumbra_sdk_proto::{core::transaction::v1 as pb, DomainType};

use crate::{ActionView, IsAction, TransactionPerspective};
use serde::{Deserialize, Serialize};

/// An action performed by a Penumbra transaction.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::Action", into = "pb::Action")]
#[allow(clippy::large_enum_variant)]
pub enum Action {
    Output(penumbra_sdk_shielded_pool::Output),
    Spend(penumbra_sdk_shielded_pool::Spend),
    ValidatorDefinition(penumbra_sdk_stake::validator::Definition),
    IbcRelay(penumbra_sdk_ibc::IbcRelay),
    Swap(penumbra_sdk_dex::swap::Swap),
    SwapClaim(penumbra_sdk_dex::swap_claim::SwapClaim),
    ProposalSubmit(penumbra_sdk_governance::ProposalSubmit),
    ProposalWithdraw(penumbra_sdk_governance::ProposalWithdraw),
    DelegatorVote(penumbra_sdk_governance::DelegatorVote),
    ValidatorVote(penumbra_sdk_governance::ValidatorVote),
    ProposalDepositClaim(penumbra_sdk_governance::ProposalDepositClaim),

    PositionOpen(penumbra_sdk_dex::lp::action::PositionOpen),
    PositionClose(penumbra_sdk_dex::lp::action::PositionClose),
    PositionWithdraw(penumbra_sdk_dex::lp::action::PositionWithdraw),

    Delegate(penumbra_sdk_stake::Delegate),
    Undelegate(penumbra_sdk_stake::Undelegate),
    UndelegateClaim(penumbra_sdk_stake::UndelegateClaim),

    Ics20Withdrawal(penumbra_sdk_shielded_pool::Ics20Withdrawal),

    CommunityPoolSpend(penumbra_sdk_community_pool::CommunityPoolSpend),
    CommunityPoolOutput(penumbra_sdk_community_pool::CommunityPoolOutput),
    CommunityPoolDeposit(penumbra_sdk_community_pool::CommunityPoolDeposit),

    ActionDutchAuctionSchedule(ActionDutchAuctionSchedule),
    ActionDutchAuctionEnd(ActionDutchAuctionEnd),
    ActionDutchAuctionWithdraw(ActionDutchAuctionWithdraw),
}

impl EffectingData for Action {
    fn effect_hash(&self) -> EffectHash {
        match self {
            Action::Output(output) => output.effect_hash(),
            Action::Spend(spend) => spend.effect_hash(),
            Action::Delegate(delegate) => delegate.effect_hash(),
            Action::Undelegate(undelegate) => undelegate.effect_hash(),
            Action::UndelegateClaim(claim) => claim.effect_hash(),
            Action::ProposalSubmit(submit) => submit.effect_hash(),
            Action::ProposalWithdraw(withdraw) => withdraw.effect_hash(),
            Action::ProposalDepositClaim(claim) => claim.effect_hash(),
            Action::DelegatorVote(vote) => vote.effect_hash(),
            Action::ValidatorVote(vote) => vote.effect_hash(),
            Action::SwapClaim(swap_claim) => swap_claim.effect_hash(),
            Action::Swap(swap) => swap.effect_hash(),
            Action::ValidatorDefinition(defn) => defn.effect_hash(),
            Action::IbcRelay(payload) => payload.effect_hash(),
            Action::PositionOpen(p) => p.effect_hash(),
            Action::PositionClose(p) => p.effect_hash(),
            Action::PositionWithdraw(p) => p.effect_hash(),
            Action::Ics20Withdrawal(w) => w.effect_hash(),
            Action::CommunityPoolSpend(d) => d.effect_hash(),
            Action::CommunityPoolOutput(d) => d.effect_hash(),
            Action::CommunityPoolDeposit(d) => d.effect_hash(),
            Action::ActionDutchAuctionSchedule(a) => a.effect_hash(),
            Action::ActionDutchAuctionEnd(a) => a.effect_hash(),
            Action::ActionDutchAuctionWithdraw(a) => a.effect_hash(),
        }
    }
}

impl Action {
    /// Create a tracing span to track execution related to this action.
    ///
    /// The `idx` parameter is the index of this action in the transaction.
    pub fn create_span(&self, idx: usize) -> tracing::Span {
        match self {
            Action::Output(_) => tracing::info_span!("Output", ?idx),
            Action::Spend(_) => tracing::info_span!("Spend", ?idx),
            Action::ValidatorDefinition(_) => {
                tracing::info_span!("ValidatorDefinition", ?idx)
            }
            Action::IbcRelay(msg) => {
                // Construct a nested span, identifying the IbcAction within
                // the transaction but also the message within the IbcAction.
                let action_span = tracing::info_span!("IbcAction", ?idx);
                msg.create_span(&action_span)
            }
            Action::Swap(_) => tracing::info_span!("Swap", ?idx),
            Action::SwapClaim(_) => tracing::info_span!("SwapClaim", ?idx),
            Action::ProposalSubmit(_) => tracing::info_span!("ProposalSubmit", ?idx),
            Action::ProposalWithdraw(_) => {
                tracing::info_span!("ProposalWithdraw", ?idx)
            }
            Action::DelegatorVote(_) => tracing::info_span!("DelegatorVote", ?idx),
            Action::ValidatorVote(_) => tracing::info_span!("ValidatorVote", ?idx),
            Action::ProposalDepositClaim(_) => {
                tracing::info_span!("ProposalDepositClaim", ?idx)
            }
            Action::PositionOpen(_) => tracing::info_span!("PositionOpen", ?idx),
            Action::PositionClose(_) => tracing::info_span!("PositionClose", ?idx),
            Action::PositionWithdraw(_) => {
                tracing::info_span!("PositionWithdraw", ?idx)
            }
            Action::Delegate(_) => tracing::info_span!("Delegate", ?idx),
            Action::Undelegate(_) => tracing::info_span!("Undelegate", ?idx),
            Action::UndelegateClaim(_) => tracing::info_span!("UndelegateClaim", ?idx),
            Action::Ics20Withdrawal(_) => tracing::info_span!("Ics20Withdrawal", ?idx),
            Action::CommunityPoolDeposit(_) => tracing::info_span!("CommunityPoolDeposit", ?idx),
            Action::CommunityPoolSpend(_) => tracing::info_span!("CommunityPoolSpend", ?idx),
            Action::CommunityPoolOutput(_) => tracing::info_span!("CommunityPoolOutput", ?idx),
            Action::ActionDutchAuctionSchedule(_) => {
                tracing::info_span!("ActionDutchAuctionSchedule", ?idx)
            }
            Action::ActionDutchAuctionEnd(_) => tracing::info_span!("ActionDutchAuctionEnd", ?idx),
            Action::ActionDutchAuctionWithdraw(_) => {
                tracing::info_span!("ActionDutchAuctionWithdraw", ?idx)
            }
        }
    }

    /// Canonical action ordering according to protobuf definitions
    pub fn variant_index(&self) -> usize {
        match self {
            Action::Spend(_) => 1,
            Action::Output(_) => 2,
            Action::Swap(_) => 3,
            Action::SwapClaim(_) => 4,
            Action::ValidatorDefinition(_) => 16,
            Action::IbcRelay(_) => 17,
            Action::ProposalSubmit(_) => 18,
            Action::ProposalWithdraw(_) => 19,
            Action::ValidatorVote(_) => 20,
            Action::DelegatorVote(_) => 21,
            Action::ProposalDepositClaim(_) => 22,
            Action::PositionOpen(_) => 30,
            Action::PositionClose(_) => 31,
            Action::PositionWithdraw(_) => 32,
            Action::Delegate(_) => 40,
            Action::Undelegate(_) => 41,
            Action::UndelegateClaim(_) => 42,
            Action::CommunityPoolSpend(_) => 50,
            Action::CommunityPoolOutput(_) => 51,
            Action::CommunityPoolDeposit(_) => 52,
            Action::Ics20Withdrawal(_) => 200,
            Action::ActionDutchAuctionSchedule(_) => 53,
            Action::ActionDutchAuctionEnd(_) => 54,
            Action::ActionDutchAuctionWithdraw(_) => 55,
        }
    }
}

impl IsAction for Action {
    fn balance_commitment(&self) -> balance::Commitment {
        match self {
            Action::Output(output) => output.balance_commitment(),
            Action::Spend(spend) => spend.balance_commitment(),
            Action::Delegate(delegate) => delegate.balance_commitment(),
            Action::Undelegate(undelegate) => undelegate.balance_commitment(),
            Action::UndelegateClaim(undelegate_claim) => undelegate_claim.balance_commitment(),
            Action::Swap(swap) => swap.balance_commitment(),
            Action::SwapClaim(swap_claim) => swap_claim.balance_commitment(),
            Action::ProposalSubmit(submit) => submit.balance_commitment(),
            Action::ProposalWithdraw(withdraw) => withdraw.balance_commitment(),
            Action::DelegatorVote(delegator_vote) => delegator_vote.balance_commitment(),
            Action::ValidatorVote(validator_vote) => validator_vote.balance_commitment(),
            Action::ProposalDepositClaim(p) => p.balance_commitment(),
            Action::PositionOpen(p) => p.balance_commitment(),
            Action::PositionClose(p) => p.balance_commitment(),
            Action::PositionWithdraw(p) => p.balance_commitment(),
            Action::Ics20Withdrawal(withdrawal) => withdrawal.balance_commitment(),
            Action::CommunityPoolDeposit(deposit) => deposit.balance_commitment(),
            Action::CommunityPoolSpend(spend) => spend.balance_commitment(),
            Action::CommunityPoolOutput(output) => output.balance_commitment(),
            // These actions just post Protobuf data to the chain, and leave the
            // value balance unchanged.
            Action::IbcRelay(x) => x.balance_commitment(),
            Action::ValidatorDefinition(_) => balance::Commitment::default(),
            Action::ActionDutchAuctionSchedule(action) => action.balance_commitment(),
            Action::ActionDutchAuctionEnd(action) => action.balance_commitment(),
            Action::ActionDutchAuctionWithdraw(action) => action.balance_commitment(),
        }
    }

    fn view_from_perspective(&self, txp: &TransactionPerspective) -> ActionView {
        match self {
            Action::Swap(x) => x.view_from_perspective(txp),
            Action::SwapClaim(x) => x.view_from_perspective(txp),
            Action::Output(x) => x.view_from_perspective(txp),
            Action::Spend(x) => x.view_from_perspective(txp),
            Action::Delegate(x) => x.view_from_perspective(txp),
            Action::Undelegate(x) => x.view_from_perspective(txp),
            Action::UndelegateClaim(x) => x.view_from_perspective(txp),
            Action::ProposalSubmit(x) => x.view_from_perspective(txp),
            Action::ProposalWithdraw(x) => x.view_from_perspective(txp),
            Action::DelegatorVote(x) => x.view_from_perspective(txp),
            Action::ValidatorVote(x) => x.view_from_perspective(txp),
            Action::ProposalDepositClaim(x) => x.view_from_perspective(txp),
            Action::PositionOpen(x) => x.view_from_perspective(txp),
            Action::PositionClose(x) => x.view_from_perspective(txp),
            Action::PositionWithdraw(x) => x.view_from_perspective(txp),
            Action::Ics20Withdrawal(x) => x.view_from_perspective(txp),
            Action::CommunityPoolSpend(x) => x.view_from_perspective(txp),
            Action::CommunityPoolOutput(x) => x.view_from_perspective(txp),
            Action::CommunityPoolDeposit(x) => x.view_from_perspective(txp),
            Action::ValidatorDefinition(x) => ActionView::ValidatorDefinition(x.to_owned()),
            Action::IbcRelay(x) => ActionView::IbcRelay(x.to_owned()),
            Action::ActionDutchAuctionSchedule(x) => x.view_from_perspective(txp),
            Action::ActionDutchAuctionEnd(x) => x.view_from_perspective(txp),
            Action::ActionDutchAuctionWithdraw(x) => x.view_from_perspective(txp),
        }
    }
}

impl DomainType for Action {
    type Proto = pb::Action;
}

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
            Action::UndelegateClaim(inner) => pb::Action {
                action: Some(pb::action::Action::UndelegateClaim(inner.into())),
            },
            Action::ValidatorDefinition(inner) => pb::Action {
                action: Some(pb::action::Action::ValidatorDefinition(inner.into())),
            },
            Action::SwapClaim(inner) => pb::Action {
                action: Some(pb::action::Action::SwapClaim(inner.into())),
            },
            Action::Swap(inner) => pb::Action {
                action: Some(pb::action::Action::Swap(inner.into())),
            },
            Action::IbcRelay(inner) => pb::Action {
                action: Some(pb::action::Action::IbcRelayAction(inner.into())),
            },
            Action::ProposalSubmit(inner) => pb::Action {
                action: Some(pb::action::Action::ProposalSubmit(inner.into())),
            },
            Action::ProposalWithdraw(inner) => pb::Action {
                action: Some(pb::action::Action::ProposalWithdraw(inner.into())),
            },
            Action::DelegatorVote(inner) => pb::Action {
                action: Some(pb::action::Action::DelegatorVote(inner.into())),
            },
            Action::ValidatorVote(inner) => pb::Action {
                action: Some(pb::action::Action::ValidatorVote(inner.into())),
            },
            Action::ProposalDepositClaim(inner) => pb::Action {
                action: Some(pb::action::Action::ProposalDepositClaim(inner.into())),
            },
            Action::PositionOpen(inner) => pb::Action {
                action: Some(pb::action::Action::PositionOpen(inner.into())),
            },
            Action::PositionClose(inner) => pb::Action {
                action: Some(pb::action::Action::PositionClose(inner.into())),
            },
            Action::PositionWithdraw(inner) => pb::Action {
                action: Some(pb::action::Action::PositionWithdraw(inner.into())),
            },
            Action::Ics20Withdrawal(withdrawal) => pb::Action {
                action: Some(pb::action::Action::Ics20Withdrawal(withdrawal.into())),
            },
            Action::CommunityPoolSpend(inner) => pb::Action {
                action: Some(pb::action::Action::CommunityPoolSpend(inner.into())),
            },
            Action::CommunityPoolOutput(inner) => pb::Action {
                action: Some(pb::action::Action::CommunityPoolOutput(inner.into())),
            },
            Action::CommunityPoolDeposit(inner) => pb::Action {
                action: Some(pb::action::Action::CommunityPoolDeposit(inner.into())),
            },
            Action::ActionDutchAuctionSchedule(inner) => pb::Action {
                action: Some(pb::action::Action::ActionDutchAuctionSchedule(inner.into())),
            },
            Action::ActionDutchAuctionEnd(inner) => pb::Action {
                action: Some(pb::action::Action::ActionDutchAuctionEnd(inner.into())),
            },
            Action::ActionDutchAuctionWithdraw(inner) => pb::Action {
                action: Some(pb::action::Action::ActionDutchAuctionWithdraw(inner.into())),
            },
        }
    }
}

impl TryFrom<pb::Action> for Action {
    type Error = anyhow::Error;
    fn try_from(proto: pb::Action) -> anyhow::Result<Self, Self::Error> {
        if proto.action.is_none() {
            anyhow::bail!("missing action content");
        }
        match proto
            .action
            .ok_or_else(|| anyhow!("missing action in Action protobuf"))?
        {
            pb::action::Action::Output(inner) => Ok(Action::Output(inner.try_into()?)),
            pb::action::Action::Spend(inner) => Ok(Action::Spend(inner.try_into()?)),
            pb::action::Action::Delegate(inner) => Ok(Action::Delegate(inner.try_into()?)),
            pb::action::Action::Undelegate(inner) => Ok(Action::Undelegate(inner.try_into()?)),
            pb::action::Action::UndelegateClaim(inner) => {
                Ok(Action::UndelegateClaim(inner.try_into()?))
            }
            pb::action::Action::ValidatorDefinition(inner) => {
                Ok(Action::ValidatorDefinition(inner.try_into()?))
            }
            pb::action::Action::SwapClaim(inner) => Ok(Action::SwapClaim(inner.try_into()?)),
            pb::action::Action::Swap(inner) => Ok(Action::Swap(inner.try_into()?)),
            pb::action::Action::IbcRelayAction(inner) => Ok(Action::IbcRelay(inner.try_into()?)),
            pb::action::Action::ProposalSubmit(inner) => {
                Ok(Action::ProposalSubmit(inner.try_into()?))
            }
            pb::action::Action::ProposalWithdraw(inner) => {
                Ok(Action::ProposalWithdraw(inner.try_into()?))
            }
            pb::action::Action::DelegatorVote(inner) => {
                Ok(Action::DelegatorVote(inner.try_into()?))
            }
            pb::action::Action::ValidatorVote(inner) => {
                Ok(Action::ValidatorVote(inner.try_into()?))
            }
            pb::action::Action::ProposalDepositClaim(inner) => {
                Ok(Action::ProposalDepositClaim(inner.try_into()?))
            }

            pb::action::Action::PositionOpen(inner) => Ok(Action::PositionOpen(inner.try_into()?)),
            pb::action::Action::PositionClose(inner) => {
                Ok(Action::PositionClose(inner.try_into()?))
            }
            pb::action::Action::PositionWithdraw(inner) => {
                Ok(Action::PositionWithdraw(inner.try_into()?))
            }
            pb::action::Action::PositionRewardClaim(_) => {
                Err(anyhow!("PositionRewardClaim is deprecated and unsupported"))
            }
            pb::action::Action::Ics20Withdrawal(inner) => {
                Ok(Action::Ics20Withdrawal(inner.try_into()?))
            }
            pb::action::Action::CommunityPoolSpend(inner) => {
                Ok(Action::CommunityPoolSpend(inner.try_into()?))
            }
            pb::action::Action::CommunityPoolOutput(inner) => {
                Ok(Action::CommunityPoolOutput(inner.try_into()?))
            }
            pb::action::Action::CommunityPoolDeposit(inner) => {
                Ok(Action::CommunityPoolDeposit(inner.try_into()?))
            }
            pb::action::Action::ActionDutchAuctionSchedule(inner) => {
                Ok(Action::ActionDutchAuctionSchedule(inner.try_into()?))
            }
            pb::action::Action::ActionDutchAuctionEnd(inner) => {
                Ok(Action::ActionDutchAuctionEnd(inner.try_into()?))
            }
            pb::action::Action::ActionDutchAuctionWithdraw(inner) => {
                Ok(Action::ActionDutchAuctionWithdraw(inner.try_into()?))
            }
        }
    }
}
