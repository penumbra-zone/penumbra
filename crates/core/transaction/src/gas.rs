use penumbra_chain::params::ChainParameters;
use penumbra_community_pool::{CommunityPoolDeposit, CommunityPoolOutput, CommunityPoolSpend};
use penumbra_dex::{
    PositionClose, PositionOpen, PositionRewardClaim, PositionWithdraw, Swap, SwapClaim,
};
use penumbra_fee::Gas;
use penumbra_ibc::IbcRelay;
use penumbra_shielded_pool::{Ics20Withdrawal, Output, Spend};
use penumbra_stake::{
    validator::Definition as ValidatorDefinition, Delegate, Undelegate, UndelegateClaim,
};

use penumbra_governance::{
    DelegatorVote, ProposalDepositClaim, ProposalKind, ProposalSubmit, ProposalWithdraw,
    ValidatorVote,
};

use crate::{
    plan::{ActionPlan, TransactionPlan},
    Action, Transaction,
};

const NULLIFIER_SIZE: u64 = 2 + 32;
const NOTEPAYLOAD_SIZE: u64 = 2 + 32 + 2 + 32 + 2 + 132;
const SWAPPAYLOAD_SIZE: u64 = 2 + 32 + 2 + 272;
// This is an approximation, the actual size is variable
const BSOD_SIZE: u64 = 16 + 16 + 0 + 4 + 64 + 4;

/// Allows [`Action`]s and [`Transaction`]s to statically indicate their relative resource consumption.
/// Since the gas cost needs to be multiplied by a price, the values returned
/// only need to be scaled relatively to each other.
pub trait GasCost {
    fn gas_cost(&self) -> Gas;
}

fn spend_gas_cost() -> Gas {
    Gas {
        // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
        // will use the encoded size of the complete transaction to calculate the block space.
        block_space: 0,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a Spend this is the byte size of a `Nullifier`.
        compact_block_space: NULLIFIER_SIZE,
        // Includes a zk-SNARK proof, so we include a constant verification cost.
        verification: 1000,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn output_gas_cost() -> Gas {
    Gas {
        // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
        // will use the encoded size of the complete transaction to calculate the block space.
        block_space: 0,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        compact_block_space: NOTEPAYLOAD_SIZE,
        // Includes a zk-SNARK proof, so we include a constant verification cost.
        verification: 1000,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn delegate_gas_cost() -> Gas {
    Gas {
        // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
        // will use the encoded size of the complete transaction to calculate the block space.
        block_space: 0,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a Delegate, nothing is added to the compact block directly. The associated [`Action::Spend`]
        // actions will add their costs, but there's nothing to add here.
        compact_block_space: 0u64,
        // Does not include a zk-SNARK proof, so there's no verification cost.
        verification: 0,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn undelegate_gas_cost() -> Gas {
    Gas {
        // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
        // will use the encoded size of the complete transaction to calculate the block space.
        block_space: 0,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For an Undelegate, nothing is added to the compact block directly. The associated [`Action::Spend`]
        // actions will add their costs, but there's nothing to add here.
        compact_block_space: 0u64,
        // Does not include a zk-SNARK proof, so there's no verification cost.
        verification: 0,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn undelegate_claim_gas_cost() -> Gas {
    Gas {
        // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
        // will use the encoded size of the complete transaction to calculate the block space.
        block_space: 0,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For an UndelegateClaim, nothing is added to the compact block directly. The associated [`Action::Output`]
        // actions will add their costs, but there's nothing to add here.
        compact_block_space: 0,
        // Includes a zk-SNARK proof, so we include a constant verification cost.
        verification: 1000,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn validator_definition_gas_cost() -> Gas {
    Gas {
        // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
        // will use the encoded size of the complete transaction to calculate the block space.
        block_space: 0,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a ValidatorDefinition the compact block is not modified.
        compact_block_space: 0u64,
        // Includes a signature verification, so we include a small constant verification cost.
        verification: 200,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn swap_gas_cost() -> Gas {
    Gas {
        // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
        // will use the encoded size of the complete transaction to calculate the block space.
        block_space: 0,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a Swap this is the byte size of a [`StatePayload`] and a [`BatchSwapOutputData`].
        // Swaps batched so technically the cost of the `BatchSwapOutputData` is shared across
        // multiple swaps, but if only one swap for a trading pair is performed in a block, that
        // swap will add a `BatchSwapOutputData` all on its own.
        // Note: the BSOD has variable size, we pick an approximation.
        compact_block_space: SWAPPAYLOAD_SIZE + BSOD_SIZE,
        // Includes a zk-SNARK proof, so we include a constant verification cost.
        verification: 1000,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

pub fn swap_claim_gas_cost() -> Gas {
    Gas {
        // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
        // will use the encoded size of the complete transaction to calculate the block space.
        block_space: 0,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a SwapClaim, nothing is added to the compact block directly. The associated [`Action::Spend`]
        // and [`Action::Output`] actions will add their costs, but there's nothing to add here.
        compact_block_space: 0u64,
        // Includes a zk-SNARK proof, so we include a constant verification cost.
        verification: 1000,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn delegator_vote_gas_cost() -> Gas {
    Gas {
        // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
        // will use the encoded size of the complete transaction to calculate the block space.
        block_space: 0,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a DelegatorVote the compact block is not modified.
        compact_block_space: 0u64,
        // Includes a zk-SNARK proof, so we include a constant verification cost.
        verification: 1000,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn position_withdraw_gas_cost() -> Gas {
    Gas {
        // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
        // will use the encoded size of the complete transaction to calculate the block space.
        block_space: 0,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a PositionWithdraw the compact block is not modified.
        compact_block_space: 0u64,
        // Does not include a zk-SNARK proof, so there's no verification cost.
        verification: 0,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn position_reward_claim_gas_cost() -> Gas {
    Gas {
        // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
        // will use the encoded size of the complete transaction to calculate the block space.
        block_space: 0,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a PositionRewardClaim the compact block is not modified.
        compact_block_space: 0u64,
        // Does not include a zk-SNARK proof, so there's no verification cost.
        verification: 0,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

impl GasCost for Transaction {
    fn gas_cost(&self) -> Gas {
        self.actions().map(GasCost::gas_cost).sum()
    }
}

impl GasCost for TransactionPlan {
    fn gas_cost(&self) -> Gas {
        self.actions.iter().map(GasCost::gas_cost).sum()
    }
}

// The planner also needs to be able to calculate gas costs,
// however until the transaction is finalized, the planner only
// has access to `ActionPlan` variants.
//
// IMPORTANT: The results produced by this impl should always
// match what the impl for the associated `Action` variant would
// produce, otherwise the planner will not include proper gas in
// transactions.
impl GasCost for ActionPlan {
    fn gas_cost(&self) -> Gas {
        match self {
            // Some variants use separate `*Plan` inners and need their
            // own implementations; others encapsulate an `Action` variant
            // and can call the `GasCost` impl on that.
            ActionPlan::Spend(_) => spend_gas_cost(),
            ActionPlan::Output(_) => output_gas_cost(),
            ActionPlan::Delegate(d) => d.gas_cost(),
            ActionPlan::Undelegate(u) => u.gas_cost(),
            ActionPlan::UndelegateClaim(_) => undelegate_claim_gas_cost(),
            ActionPlan::ValidatorDefinition(vd) => vd.gas_cost(),
            ActionPlan::Swap(_) => swap_gas_cost(),
            ActionPlan::SwapClaim(_) => swap_claim_gas_cost(),
            ActionPlan::IbcAction(i) => i.gas_cost(),
            ActionPlan::ProposalSubmit(ps) => ps.gas_cost(),
            ActionPlan::ProposalWithdraw(pw) => pw.gas_cost(),
            ActionPlan::DelegatorVote(_) => delegator_vote_gas_cost(),
            ActionPlan::ValidatorVote(v) => v.gas_cost(),
            ActionPlan::ProposalDepositClaim(pdc) => pdc.gas_cost(),
            ActionPlan::PositionOpen(po) => po.gas_cost(),
            ActionPlan::PositionClose(pc) => pc.gas_cost(),
            ActionPlan::PositionWithdraw(_) => position_withdraw_gas_cost(),
            ActionPlan::PositionRewardClaim(_) => position_reward_claim_gas_cost(),
            ActionPlan::CommunityPoolSpend(ds) => ds.gas_cost(),
            ActionPlan::CommunityPoolOutput(d) => d.gas_cost(),
            ActionPlan::CommunityPoolDeposit(dd) => dd.gas_cost(),
            ActionPlan::Withdrawal(w) => w.gas_cost(),
        }
    }
}

impl GasCost for Action {
    fn gas_cost(&self) -> Gas {
        match self {
            Action::Output(output) => output.gas_cost(),
            Action::Spend(spend) => spend.gas_cost(),
            Action::Delegate(delegate) => delegate.gas_cost(),
            Action::Undelegate(undelegate) => undelegate.gas_cost(),
            Action::UndelegateClaim(undelegate_claim) => undelegate_claim.gas_cost(),
            Action::Swap(swap) => swap.gas_cost(),
            Action::SwapClaim(swap_claim) => swap_claim.gas_cost(),
            Action::ProposalSubmit(submit) => submit.gas_cost(),
            Action::ProposalWithdraw(withdraw) => withdraw.gas_cost(),
            Action::DelegatorVote(delegator_vote) => delegator_vote.gas_cost(),
            Action::ValidatorVote(validator_vote) => validator_vote.gas_cost(),
            Action::ProposalDepositClaim(p) => p.gas_cost(),
            Action::PositionOpen(p) => p.gas_cost(),
            Action::PositionClose(p) => p.gas_cost(),
            Action::PositionWithdraw(p) => p.gas_cost(),
            Action::PositionRewardClaim(p) => p.gas_cost(),
            Action::Ics20Withdrawal(withdrawal) => withdrawal.gas_cost(),
            Action::CommunityPoolDeposit(deposit) => deposit.gas_cost(),
            Action::CommunityPoolSpend(spend) => spend.gas_cost(),
            Action::CommunityPoolOutput(output) => output.gas_cost(),
            Action::IbcRelay(x) => x.gas_cost(),
            Action::ValidatorDefinition(x) => x.gas_cost(),
        }
    }
}

impl GasCost for Output {
    fn gas_cost(&self) -> Gas {
        output_gas_cost()
    }
}

impl GasCost for Spend {
    fn gas_cost(&self) -> Gas {
        spend_gas_cost()
    }
}

impl GasCost for Delegate {
    fn gas_cost(&self) -> Gas {
        delegate_gas_cost()
    }
}

impl GasCost for Undelegate {
    fn gas_cost(&self) -> Gas {
        undelegate_gas_cost()
    }
}

impl GasCost for UndelegateClaim {
    fn gas_cost(&self) -> Gas {
        undelegate_claim_gas_cost()
    }
}

impl GasCost for Swap {
    fn gas_cost(&self) -> Gas {
        swap_gas_cost()
    }
}

impl GasCost for SwapClaim {
    fn gas_cost(&self) -> Gas {
        swap_claim_gas_cost()
    }
}

impl GasCost for ProposalSubmit {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a ProposalSubmit the compact block is only modified if the proposal type is a `ParameterChange`.
            compact_block_space: match self.proposal.kind() {
                ProposalKind::ParameterChange => std::mem::size_of::<ChainParameters>() as u64,
                _ => 0u64,
            },
            // There are some checks performed to validate the proposed state changes, so we include a constant verification cost,
            // smaller than a zk-SNARK verification cost.
            verification: 100,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for ProposalWithdraw {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a ProposalWithdraw the compact block is not modified.
            compact_block_space: 0u64,
            // Does not include a zk-SNARK proof, so there's no verification cost.
            verification: 0,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for DelegatorVote {
    fn gas_cost(&self) -> Gas {
        delegator_vote_gas_cost()
    }
}

impl GasCost for ValidatorVote {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a ValidatorVote the compact block is not modified.
            compact_block_space: 0u64,
            // Includes a signature verification, so we include a small constant verification cost.
            verification: 200,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for ProposalDepositClaim {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a ProposalDepositClaim the compact block is not modified.
            compact_block_space: 0u64,
            // Does not include a zk-SNARK proof, so there's no verification cost.
            verification: 0,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for PositionOpen {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a PositionOpen the compact block is not modified.
            compact_block_space: 0u64,
            // There are some small validations performed so a token amount of gas is charged.
            verification: 50,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for PositionClose {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a PositionClose the compact block is not modified.
            compact_block_space: 0u64,
            // Does not include a zk-SNARK proof, so there's no verification cost.
            verification: 0,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for PositionWithdraw {
    fn gas_cost(&self) -> Gas {
        position_withdraw_gas_cost()
    }
}

impl GasCost for PositionRewardClaim {
    fn gas_cost(&self) -> Gas {
        position_reward_claim_gas_cost()
    }
}

impl GasCost for Ics20Withdrawal {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a Ics20Withdrawal the compact block is not modified.
            compact_block_space: 0u64,
            // Does not include a zk-SNARK proof, so there's no verification cost.
            verification: 0,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for CommunityPoolDeposit {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a CommunityPoolDeposit the compact block is not modified.
            compact_block_space: 0u64,
            // Does not include a zk-SNARK proof, so there's no verification cost.
            verification: 0,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for CommunityPoolSpend {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a CommunityPoolSpend the compact block is not modified.
            compact_block_space: 0u64,
            // Does not include a zk-SNARK proof, so there's no verification cost.
            verification: 0,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for CommunityPoolOutput {
    fn gas_cost(&self) -> Gas {
        // We hardcode the gas costs of a `CommunityPoolOutput` to 0, since it's a protocol action.
        Gas {
            block_space: 0,
            compact_block_space: 0,
            verification: 0,
            execution: 0,
        }
    }
}

impl GasCost for IbcRelay {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a IbcAction this is the byte size of a [`StatePayload`].
            compact_block_space: match self {
                // RecvPacket will mint a note if successful.
                IbcRelay::RecvPacket(_) => NOTEPAYLOAD_SIZE,
                _ => 0u64,
            },
            // Includes a proof in the execution for RecvPacket (TODO: check the other variants).
            verification: match self {
                IbcRelay::RecvPacket(_) => 1000,
                _ => 0u64,
            },
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for ValidatorDefinition {
    fn gas_cost(&self) -> Gas {
        validator_definition_gas_cost()
    }
}
