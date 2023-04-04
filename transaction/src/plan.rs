//! Declarative transaction plans, used for transaction authorization and
//! creation.

use anyhow::Result;
use penumbra_crypto::{transaction::Fee, Address};
use penumbra_proto::{
    core::stake::v1alpha1 as pb_stake, core::transaction::v1alpha1 as pb, DomainType,
};
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};

use crate::action::{
    DaoDeposit, DaoOutput, DaoSpend, Delegate, IbcAction, PositionClose, PositionOpen,
    ProposalDepositClaim, ProposalSubmit, ProposalWithdraw, Undelegate, ValidatorVote,
};

mod action;
mod auth;
mod build;
mod clue;
mod memo;

pub use action::{
    ActionPlan, DelegatorVotePlan, Ics20WithdrawalPlan, OutputPlan, PositionRewardClaimPlan,
    PositionWithdrawPlan, SpendPlan, SwapClaimPlan, SwapPlan, UndelegateClaimPlan,
};
pub use clue::CluePlan;
pub use memo::MemoPlan;

/// A declaration of a planned [`Transaction`](crate::Transaction),
/// for use in transaction authorization and creation.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "pb::TransactionPlan", into = "pb::TransactionPlan")]
pub struct TransactionPlan {
    /// A list of this transaction's actions.
    pub actions: Vec<ActionPlan>,
    pub expiry_height: u64,
    pub chain_id: String,
    pub fee: Fee,
    pub clue_plans: Vec<CluePlan>,
    pub memo_plan: Option<MemoPlan>,
}

impl TransactionPlan {
    pub fn spend_plans(&self) -> impl Iterator<Item = &SpendPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::Spend(s) = action {
                Some(s)
            } else {
                None
            }
        })
    }

    pub fn output_plans(&self) -> impl Iterator<Item = &OutputPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::Output(o) = action {
                Some(o)
            } else {
                None
            }
        })
    }

    pub fn clue_plans(&self) -> impl Iterator<Item = &CluePlan> {
        self.clue_plans.iter()
    }

    pub fn delegations(&self) -> impl Iterator<Item = &Delegate> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::Delegate(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn undelegations(&self) -> impl Iterator<Item = &Undelegate> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::Undelegate(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn undelegate_claim_plans(&self) -> impl Iterator<Item = &UndelegateClaimPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::UndelegateClaim(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn ibc_actions(&self) -> impl Iterator<Item = &IbcAction> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::IbcAction(ibc_action) = action {
                Some(ibc_action)
            } else {
                None
            }
        })
    }

    pub fn validator_definitions(&self) -> impl Iterator<Item = &pb_stake::ValidatorDefinition> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::ValidatorDefinition(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn proposal_submits(&self) -> impl Iterator<Item = &ProposalSubmit> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::ProposalSubmit(p) = action {
                Some(p)
            } else {
                None
            }
        })
    }

    pub fn proposal_withdraws(&self) -> impl Iterator<Item = &ProposalWithdraw> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::ProposalWithdraw(p) = action {
                Some(p)
            } else {
                None
            }
        })
    }

    pub fn delegator_vote_plans(&self) -> impl Iterator<Item = &DelegatorVotePlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::DelegatorVote(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn validator_votes(&self) -> impl Iterator<Item = &ValidatorVote> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::ValidatorVote(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn proposal_deposit_claims(&self) -> impl Iterator<Item = &ProposalDepositClaim> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::ProposalDepositClaim(p) = action {
                Some(p)
            } else {
                None
            }
        })
    }

    pub fn swap_plans(&self) -> impl Iterator<Item = &SwapPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::Swap(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn swap_claim_plans(&self) -> impl Iterator<Item = &SwapClaimPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::SwapClaim(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn dao_spends(&self) -> impl Iterator<Item = &DaoSpend> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::DaoSpend(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn dao_deposits(&self) -> impl Iterator<Item = &DaoDeposit> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::DaoDeposit(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn dao_outputs(&self) -> impl Iterator<Item = &DaoOutput> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::DaoOutput(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn position_openings(&self) -> impl Iterator<Item = &PositionOpen> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::PositionOpen(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn position_closings(&self) -> impl Iterator<Item = &PositionClose> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::PositionClose(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn position_withdrawals(&self) -> impl Iterator<Item = &PositionWithdrawPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::PositionWithdraw(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn ics20_withdrawals(&self) -> impl Iterator<Item = &Ics20WithdrawalPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::WithdrawalPlan(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    /// Convenience method to get all the destination addresses for each `OutputPlan`s.
    pub fn dest_addresses(&self) -> Vec<Address> {
        self.output_plans()
            .into_iter()
            .map(|plan| plan.dest_address)
            .collect()
    }

    /// Convenience method to get the number of `OutputPlan`s in this transaction.
    pub fn num_outputs(&self) -> usize {
        self.output_plans().into_iter().count()
    }

    /// Method to add `CluePlan`s to a `TransactionPlan`.
    pub fn add_all_clue_plans<R: CryptoRng + Rng>(&mut self, mut rng: R, precision_bits: usize) {
        // Add one clue per recipient.
        let mut clue_plans = vec![];
        for dest_address in self.dest_addresses() {
            clue_plans.push(CluePlan::new(&mut rng, dest_address, precision_bits));
        }

        // Now add dummy clues until we have one clue per output.
        let num_dummy_clues = self.num_outputs() - clue_plans.len();
        for _ in 0..num_dummy_clues {
            let dummy_address = Address::dummy(&mut rng);
            clue_plans.push(CluePlan::new(&mut rng, dummy_address, precision_bits));
        }

        self.clue_plans = clue_plans;
    }
}

impl DomainType for TransactionPlan {
    type Proto = pb::TransactionPlan;
}

impl From<TransactionPlan> for pb::TransactionPlan {
    fn from(msg: TransactionPlan) -> Self {
        Self {
            actions: msg.actions.into_iter().map(Into::into).collect(),
            expiry_height: msg.expiry_height,
            chain_id: msg.chain_id,
            fee: Some(msg.fee.into()),
            clue_plans: msg.clue_plans.into_iter().map(Into::into).collect(),
            memo_plan: msg.memo_plan.map(Into::into),
        }
    }
}

impl TryFrom<pb::TransactionPlan> for TransactionPlan {
    type Error = anyhow::Error;
    fn try_from(value: pb::TransactionPlan) -> Result<Self, Self::Error> {
        let memo_plan = match value.memo_plan {
            Some(plan) => Some(plan.try_into()?),
            None => None,
        };

        Ok(Self {
            actions: value
                .actions
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            expiry_height: value.expiry_height,
            chain_id: value.chain_id,
            fee: value
                .fee
                .ok_or_else(|| anyhow::anyhow!("missing fee"))?
                .try_into()?,
            clue_plans: value
                .clue_plans
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            memo_plan,
        })
    }
}
