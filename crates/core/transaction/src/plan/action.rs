use crate::Action;
use crate::WitnessData;
use anyhow::{anyhow, Context, Result};
use ark_ff::Zero;
use decaf377::Fr;
use penumbra_sdk_asset::Balance;
use penumbra_sdk_auction::auction::dutch::actions::ActionDutchAuctionEnd;
use penumbra_sdk_auction::auction::dutch::actions::ActionDutchAuctionSchedule;
use penumbra_sdk_auction::auction::dutch::actions::ActionDutchAuctionWithdrawPlan;
use penumbra_sdk_community_pool::{CommunityPoolDeposit, CommunityPoolOutput, CommunityPoolSpend};
use penumbra_sdk_dex::PositionOpen;
use penumbra_sdk_dex::{
    lp::{
        action::PositionClose,
        plan::{PositionOpenPlan, PositionWithdrawPlan},
    },
    swap::SwapPlan,
    swap_claim::SwapClaimPlan,
};
use penumbra_sdk_funding::liquidity_tournament::ActionLiquidityTournamentVotePlan;
use penumbra_sdk_governance::{
    delegator_vote::DelegatorVotePlan, ProposalDepositClaim, ProposalSubmit, ProposalWithdraw,
    ValidatorVote,
};
use penumbra_sdk_txhash::{EffectHash, EffectingData};

use penumbra_sdk_ibc::IbcRelay;
use penumbra_sdk_keys::{symmetric::PayloadKey, FullViewingKey};
use penumbra_sdk_proto::{core::transaction::v1 as pb_t, DomainType};
use penumbra_sdk_shielded_pool::{Ics20Withdrawal, OutputPlan, SpendPlan};
use penumbra_sdk_stake::{Delegate, Undelegate, UndelegateClaimPlan};
use serde::{Deserialize, Serialize};

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
    ValidatorDefinition(penumbra_sdk_stake::validator::Definition),
    /// Describes a proposed swap.
    Swap(SwapPlan),
    /// Describes a swap claim.
    SwapClaim(SwapClaimPlan),
    IbcAction(IbcRelay),
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
    /// We need a full plan here, because the metadata will be encrypted.
    PositionOpen(PositionOpenPlan),
    PositionClose(PositionClose),
    // PositionWithdrawPlan requires the balance of the funds to be withdrawn, so
    // a plan must be used.
    PositionWithdraw(PositionWithdrawPlan),

    CommunityPoolSpend(CommunityPoolSpend),
    CommunityPoolOutput(CommunityPoolOutput),
    CommunityPoolDeposit(CommunityPoolDeposit),

    Ics20Withdrawal(Ics20Withdrawal),

    ActionDutchAuctionSchedule(ActionDutchAuctionSchedule),
    ActionDutchAuctionEnd(ActionDutchAuctionEnd),
    ActionDutchAuctionWithdraw(ActionDutchAuctionWithdrawPlan),

    ActionLiquidityTournamentVote(ActionLiquidityTournamentVotePlan),
}

impl ActionPlan {
    /// Builds a planned [`Action`] specified by this [`ActionPlan`].
    ///
    /// The resulting action is `unauth` in the sense that this method does not
    /// have access to authorization data, so any required authorization data
    /// will be filled in with dummy zero values, to be replaced later.
    ///
    /// This method is useful for controlling how a transaction's actions are
    /// built (e.g., building them in parallel, or via Web Workers).
    pub fn build_unauth(
        action_plan: ActionPlan,
        fvk: &FullViewingKey,
        witness_data: &WitnessData,
        memo_key: Option<PayloadKey>,
    ) -> Result<Action> {
        use ActionPlan::*;

        Ok(match action_plan {
            Spend(spend_plan) => {
                let note_commitment = spend_plan.note.commit();
                let auth_path = witness_data
                    .state_commitment_proofs
                    .get(&note_commitment)
                    .context(format!("could not get proof for {note_commitment:?}"))?;

                Action::Spend(spend_plan.spend(
                    fvk,
                    [0; 64].into(),
                    auth_path.clone(),
                    // FIXME: why does this need the anchor? isn't that implied by the auth_path?
                    // cf. delegator_vote
                    witness_data.anchor,
                ))
            }
            Output(output_plan) => {
                let dummy_payload_key: PayloadKey = [0u8; 32].into();
                Action::Output(output_plan.output(
                    fvk.outgoing(),
                    memo_key.as_ref().unwrap_or(&dummy_payload_key),
                ))
            }
            Swap(swap_plan) => Action::Swap(swap_plan.swap(fvk)),
            SwapClaim(swap_claim_plan) => {
                let note_commitment = swap_claim_plan.swap_plaintext.swap_commitment();
                let auth_path = witness_data
                    .state_commitment_proofs
                    .get(&note_commitment)
                    .context(format!("could not get proof for {note_commitment:?}"))?;

                Action::SwapClaim(swap_claim_plan.swap_claim(fvk, auth_path))
            }
            Delegate(plan) => Action::Delegate(plan.clone()),
            Undelegate(plan) => Action::Undelegate(plan.clone()),
            UndelegateClaim(plan) => Action::UndelegateClaim(plan.undelegate_claim()),
            ValidatorDefinition(plan) => Action::ValidatorDefinition(plan.clone()),
            // Fixme: action name
            IbcAction(plan) => Action::IbcRelay(plan.clone()),
            ProposalSubmit(plan) => Action::ProposalSubmit(plan.clone()),
            ProposalWithdraw(plan) => Action::ProposalWithdraw(plan.clone()),
            DelegatorVote(plan) => {
                let note_commitment = plan.staked_note.commit();
                let auth_path = witness_data
                    .state_commitment_proofs
                    .get(&note_commitment)
                    .context(format!("could not get proof for {note_commitment:?}"))?;
                Action::DelegatorVote(plan.delegator_vote(fvk, [0; 64].into(), auth_path.clone()))
            }
            ValidatorVote(plan) => Action::ValidatorVote(plan.clone()),
            ProposalDepositClaim(plan) => Action::ProposalDepositClaim(plan.clone()),
            PositionOpen(plan) => Action::PositionOpen(plan.position_open(fvk, None)),
            PositionClose(plan) => Action::PositionClose(plan.clone()),
            PositionWithdraw(plan) => Action::PositionWithdraw(plan.position_withdraw()),
            CommunityPoolSpend(plan) => Action::CommunityPoolSpend(plan.clone()),
            CommunityPoolOutput(plan) => Action::CommunityPoolOutput(plan.clone()),
            CommunityPoolDeposit(plan) => Action::CommunityPoolDeposit(plan.clone()),
            Ics20Withdrawal(plan) => Action::Ics20Withdrawal(plan.clone()),
            ActionDutchAuctionSchedule(plan) => Action::ActionDutchAuctionSchedule(plan.clone()),
            ActionDutchAuctionEnd(plan) => Action::ActionDutchAuctionEnd(plan.clone()),
            ActionDutchAuctionWithdraw(plan) => {
                Action::ActionDutchAuctionWithdraw(plan.to_action())
            }
            ActionLiquidityTournamentVote(plan) => {
                let note_commitment = plan.staked_note.commit();
                let auth_path = witness_data
                    .state_commitment_proofs
                    .get(&note_commitment)
                    .context(format!("could not get proof for {note_commitment:?}"))?;
                Action::ActionLiquidityTournamentVote(plan.to_action(
                    fvk,
                    [0; 64].into(),
                    auth_path.clone(),
                ))
            }
        })
    }

    /// Canonical action plan ordering according to protobuf definitions
    pub fn variant_index(&self) -> usize {
        match self {
            ActionPlan::Spend(_) => 1,
            ActionPlan::Output(_) => 2,
            ActionPlan::Swap(_) => 3,
            ActionPlan::SwapClaim(_) => 4,
            ActionPlan::ValidatorDefinition(_) => 16,
            ActionPlan::IbcAction(_) => 17,
            ActionPlan::ProposalSubmit(_) => 18,
            ActionPlan::ProposalWithdraw(_) => 19,
            ActionPlan::ValidatorVote(_) => 20,
            ActionPlan::DelegatorVote(_) => 21,
            ActionPlan::ProposalDepositClaim(_) => 22,
            ActionPlan::PositionOpen(_) => 30,
            ActionPlan::PositionClose(_) => 31,
            ActionPlan::PositionWithdraw(_) => 32,
            ActionPlan::Delegate(_) => 40,
            ActionPlan::Undelegate(_) => 41,
            ActionPlan::UndelegateClaim(_) => 42,
            ActionPlan::CommunityPoolSpend(_) => 50,
            ActionPlan::CommunityPoolOutput(_) => 51,
            ActionPlan::CommunityPoolDeposit(_) => 52,
            ActionPlan::Ics20Withdrawal(_) => 200,
            ActionPlan::ActionDutchAuctionSchedule(_) => 53,
            ActionPlan::ActionDutchAuctionEnd(_) => 54,
            ActionPlan::ActionDutchAuctionWithdraw(_) => 55,
            ActionPlan::ActionLiquidityTournamentVote(_) => 70,
        }
    }

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
            CommunityPoolSpend(community_pool_spend) => community_pool_spend.balance(),
            CommunityPoolOutput(community_pool_output) => community_pool_output.balance(),
            CommunityPoolDeposit(community_pool_deposit) => community_pool_deposit.balance(),
            PositionOpen(position_open) => position_open.balance(),
            PositionClose(position_close) => position_close.balance(),
            PositionWithdraw(position_withdraw) => position_withdraw.balance(),
            Ics20Withdrawal(withdrawal) => withdrawal.balance(),
            ActionDutchAuctionSchedule(action) => action.balance(),
            ActionDutchAuctionEnd(action) => action.balance(),
            ActionDutchAuctionWithdraw(action) => action.balance(),

            // None of these contribute to transaction balance:
            IbcAction(_)
            | ValidatorDefinition(_)
            | ValidatorVote(_)
            | ActionLiquidityTournamentVote(_) => Balance::default(),
        }
    }

    pub fn value_blinding(&self) -> Fr {
        use ActionPlan::*;

        match self {
            Spend(spend) => spend.value_blinding,
            Output(output) => output.value_blinding,
            Delegate(_) => Fr::zero(),
            Undelegate(_) => Fr::zero(),
            UndelegateClaim(undelegate_claim) => undelegate_claim.balance_blinding,
            ValidatorDefinition(_) => Fr::zero(),
            Swap(swap) => swap.fee_blinding,
            SwapClaim(_) => Fr::zero(),
            IbcAction(_) => Fr::zero(),
            ProposalSubmit(_) => Fr::zero(),
            ProposalWithdraw(_) => Fr::zero(),
            DelegatorVote(_) => Fr::zero(),
            ValidatorVote(_) => Fr::zero(),
            ProposalDepositClaim(_) => Fr::zero(),
            PositionOpen(_) => Fr::zero(),
            PositionClose(_) => Fr::zero(),
            PositionWithdraw(_) => Fr::zero(),
            CommunityPoolSpend(_) => Fr::zero(),
            CommunityPoolOutput(_) => Fr::zero(),
            CommunityPoolDeposit(_) => Fr::zero(),
            Ics20Withdrawal(_) => Fr::zero(),
            ActionDutchAuctionSchedule(_) => Fr::zero(),
            ActionDutchAuctionEnd(_) => Fr::zero(),
            ActionDutchAuctionWithdraw(_) => Fr::zero(),
            ActionLiquidityTournamentVote(_) => Fr::zero(),
        }
    }

    /// Compute the effect hash of the action this plan will produce.
    pub fn effect_hash(&self, fvk: &FullViewingKey, memo_key: &PayloadKey) -> EffectHash {
        use ActionPlan::*;

        match self {
            Spend(plan) => plan.spend_body(fvk).effect_hash(),
            Output(plan) => plan.output_body(fvk.outgoing(), memo_key).effect_hash(),
            Delegate(plan) => plan.effect_hash(),
            Undelegate(plan) => plan.effect_hash(),
            UndelegateClaim(plan) => plan.undelegate_claim_body().effect_hash(),
            ValidatorDefinition(plan) => plan.effect_hash(),
            Swap(plan) => plan.swap_body(fvk).effect_hash(),
            SwapClaim(plan) => plan.swap_claim_body(fvk).effect_hash(),
            IbcAction(plan) => plan.effect_hash(),
            ProposalSubmit(plan) => plan.effect_hash(),
            ProposalWithdraw(plan) => plan.effect_hash(),
            DelegatorVote(plan) => plan.delegator_vote_body(fvk).effect_hash(),
            ValidatorVote(plan) => plan.effect_hash(),
            ProposalDepositClaim(plan) => plan.effect_hash(),
            PositionOpen(plan) => plan.position_open(fvk, None).effect_hash(),
            PositionClose(plan) => plan.effect_hash(),
            PositionWithdraw(plan) => plan.position_withdraw().effect_hash(),
            CommunityPoolSpend(plan) => plan.effect_hash(),
            CommunityPoolOutput(plan) => plan.effect_hash(),
            CommunityPoolDeposit(plan) => plan.effect_hash(),
            Ics20Withdrawal(plan) => plan.effect_hash(),
            ActionDutchAuctionSchedule(plan) => plan.effect_hash(),
            ActionDutchAuctionEnd(plan) => plan.effect_hash(),
            ActionDutchAuctionWithdraw(plan) => plan.to_action().effect_hash(),
            ActionLiquidityTournamentVote(plan) => plan.to_body(fvk).effect_hash(),
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

impl From<UndelegateClaimPlan> for ActionPlan {
    fn from(inner: UndelegateClaimPlan) -> ActionPlan {
        ActionPlan::UndelegateClaim(inner)
    }
}

impl From<penumbra_sdk_stake::validator::Definition> for ActionPlan {
    fn from(inner: penumbra_sdk_stake::validator::Definition) -> ActionPlan {
        ActionPlan::ValidatorDefinition(inner)
    }
}

impl From<IbcRelay> for ActionPlan {
    fn from(inner: IbcRelay) -> ActionPlan {
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

impl From<PositionOpenPlan> for ActionPlan {
    fn from(inner: PositionOpenPlan) -> ActionPlan {
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

impl From<CommunityPoolSpend> for ActionPlan {
    fn from(inner: CommunityPoolSpend) -> ActionPlan {
        ActionPlan::CommunityPoolSpend(inner)
    }
}

impl From<CommunityPoolOutput> for ActionPlan {
    fn from(inner: CommunityPoolOutput) -> ActionPlan {
        ActionPlan::CommunityPoolOutput(inner)
    }
}

impl From<CommunityPoolDeposit> for ActionPlan {
    fn from(inner: CommunityPoolDeposit) -> ActionPlan {
        ActionPlan::CommunityPoolDeposit(inner)
    }
}

impl From<Ics20Withdrawal> for ActionPlan {
    fn from(inner: Ics20Withdrawal) -> ActionPlan {
        ActionPlan::Ics20Withdrawal(inner)
    }
}

impl From<ActionDutchAuctionSchedule> for ActionPlan {
    fn from(inner: ActionDutchAuctionSchedule) -> ActionPlan {
        ActionPlan::ActionDutchAuctionSchedule(inner)
    }
}

impl From<ActionDutchAuctionEnd> for ActionPlan {
    fn from(inner: ActionDutchAuctionEnd) -> ActionPlan {
        ActionPlan::ActionDutchAuctionEnd(inner)
    }
}

impl From<ActionDutchAuctionWithdrawPlan> for ActionPlan {
    fn from(inner: ActionDutchAuctionWithdrawPlan) -> ActionPlan {
        ActionPlan::ActionDutchAuctionWithdraw(inner)
    }
}

impl From<ProposalWithdraw> for ActionPlan {
    fn from(inner: ProposalWithdraw) -> ActionPlan {
        ActionPlan::ProposalWithdraw(inner)
    }
}

impl From<ProposalDepositClaim> for ActionPlan {
    fn from(inner: ProposalDepositClaim) -> ActionPlan {
        ActionPlan::ProposalDepositClaim(inner)
    }
}

impl From<ActionLiquidityTournamentVotePlan> for ActionPlan {
    fn from(inner: ActionLiquidityTournamentVotePlan) -> ActionPlan {
        ActionPlan::ActionLiquidityTournamentVote(inner)
    }
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
                action: Some(pb_t::action_plan::Action::IbcRelayAction(inner.into())),
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
                action: Some(pb_t::action_plan::Action::PositionOpenPlan(inner.into())),
            },
            ActionPlan::PositionClose(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::PositionClose(inner.into())),
            },
            ActionPlan::PositionWithdraw(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::PositionWithdraw(Into::<
                    penumbra_sdk_proto::core::component::dex::v1::PositionWithdrawPlan,
                >::into(
                    inner
                ))),
            },
            ActionPlan::CommunityPoolDeposit(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::CommunityPoolDeposit(
                    inner.into(),
                )),
            },
            ActionPlan::CommunityPoolSpend(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::CommunityPoolSpend(inner.into())),
            },
            ActionPlan::CommunityPoolOutput(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::CommunityPoolOutput(inner.into())),
            },
            ActionPlan::Ics20Withdrawal(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::Ics20Withdrawal(inner.into())),
            },
            ActionPlan::ActionDutchAuctionSchedule(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::ActionDutchAuctionSchedule(
                    inner.into(),
                )),
            },
            ActionPlan::ActionDutchAuctionEnd(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::ActionDutchAuctionEnd(
                    inner.into(),
                )),
            },
            ActionPlan::ActionDutchAuctionWithdraw(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::ActionDutchAuctionWithdraw(
                    inner.into(),
                )),
            },
            ActionPlan::ActionLiquidityTournamentVote(inner) => pb_t::ActionPlan {
                action: Some(pb_t::action_plan::Action::ActionLiquidityTournamentVote(
                    inner.into(),
                )),
            },
        }
    }
}

impl TryFrom<pb_t::ActionPlan> for ActionPlan {
    type Error = anyhow::Error;

    fn try_from(proto: pb_t::ActionPlan) -> anyhow::Result<Self, Self::Error> {
        if proto.action.is_none() {
            anyhow::bail!("missing action content");
        }

        match proto
            .action
            .ok_or_else(|| anyhow!("missing action in ActionPlan proto"))?
        {
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
            pb_t::action_plan::Action::IbcRelayAction(inner) => {
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
            // Deprecated, but we gracefully handle it anyways.
            pb_t::action_plan::Action::PositionOpen(inner) => {
                let position_open = PositionOpen::try_from(inner)?;
                Ok(ActionPlan::PositionOpen(PositionOpenPlan {
                    position: position_open.position,
                    metadata: None,
                }))
            }
            pb_t::action_plan::Action::PositionOpenPlan(inner) => {
                Ok(ActionPlan::PositionOpen(inner.try_into()?))
            }
            pb_t::action_plan::Action::PositionClose(inner) => {
                Ok(ActionPlan::PositionClose(inner.try_into()?))
            }
            pb_t::action_plan::Action::PositionWithdraw(inner) => {
                Ok(ActionPlan::PositionWithdraw(inner.try_into()?))
            }
            pb_t::action_plan::Action::PositionRewardClaim(_) => {
                Err(anyhow!("PositionRewardClaim is deprecated and unsupported"))
            }
            pb_t::action_plan::Action::CommunityPoolSpend(inner) => {
                Ok(ActionPlan::CommunityPoolSpend(inner.try_into()?))
            }
            pb_t::action_plan::Action::CommunityPoolDeposit(inner) => {
                Ok(ActionPlan::CommunityPoolDeposit(inner.try_into()?))
            }
            pb_t::action_plan::Action::CommunityPoolOutput(inner) => {
                Ok(ActionPlan::CommunityPoolOutput(inner.try_into()?))
            }
            pb_t::action_plan::Action::ActionDutchAuctionSchedule(inner) => {
                Ok(ActionPlan::ActionDutchAuctionSchedule(inner.try_into()?))
            }
            pb_t::action_plan::Action::ActionDutchAuctionEnd(inner) => {
                Ok(ActionPlan::ActionDutchAuctionEnd(inner.try_into()?))
            }
            pb_t::action_plan::Action::ActionDutchAuctionWithdraw(inner) => {
                Ok(ActionPlan::ActionDutchAuctionWithdraw(inner.try_into()?))
            }
            pb_t::action_plan::Action::Ics20Withdrawal(inner) => {
                Ok(ActionPlan::Ics20Withdrawal(inner.try_into()?))
            }
            pb_t::action_plan::Action::ActionLiquidityTournamentVote(inner) => {
                Ok(ActionPlan::ActionLiquidityTournamentVote(inner.try_into()?))
            }
        }
    }
}
