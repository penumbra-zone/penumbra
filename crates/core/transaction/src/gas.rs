use std::ops::Add;

use penumbra_chain::params::ChainParameters;
use penumbra_compact_block::StatePayload;
use penumbra_dao::{DaoDeposit, DaoOutput, DaoSpend};
use penumbra_dex::{
    BatchSwapOutputData, PositionClose, PositionOpen, PositionRewardClaim, PositionWithdraw, Swap,
    SwapClaim,
};
use penumbra_ibc::{IbcAction, Ics20Withdrawal};
use penumbra_num::Amount;
use penumbra_proto::{core::transaction::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::{Output, Spend};
use penumbra_stake::{
    validator::Definition as ValidatorDefinition, Delegate, Undelegate, UndelegateClaim,
};

use crate::{
    action::{
        DelegatorVote, ProposalDepositClaim, ProposalKind, ProposalSubmit, ProposalWithdraw,
        ValidatorVote,
    },
    Action,
};

/// Represents the different resources that a transaction can consume,
/// for purposes of calculating multidimensional fees based on real
/// transaction resource consumption.
pub struct Gas {
    pub block_space: u64,
    pub compact_block_space: u64,
    pub verification: u64,
    pub execution: u64,
}

impl Add for Gas {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            block_space: self.block_space + rhs.block_space,
            compact_block_space: self.compact_block_space + rhs.compact_block_space,
            verification: self.verification + rhs.verification,
            execution: self.execution + rhs.execution,
        }
    }
}

/// Allows [`Action`]s and [`Transaction`]s to statically indicate their relative resource consumption.
/// Since the gas cost needs to be multiplied by a price, the values returned
/// only need to be scaled relatively to each other.
pub trait GasCost {
    fn gas_cost(&self) -> Gas;
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
            Action::DaoDeposit(deposit) => deposit.gas_cost(),
            Action::DaoSpend(spend) => spend.gas_cost(),
            Action::DaoOutput(output) => output.gas_cost(),
            Action::IbcAction(x) => x.gas_cost(),
            Action::ValidatorDefinition(x) => x.gas_cost(),
        }
    }
}

impl GasCost for Output {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For an Output this is the byte size of a [`StatePayload`].
            compact_block_space: std::mem::size_of::<StatePayload>() as u64,
            // Includes a zk-SNARK proof, so we include a constant verification cost.
            verification: 1000,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for Spend {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a Spend this is the byte size of a `Nullifier`.
            compact_block_space: std::mem::size_of::<Nullifier>() as u64,
            // Includes a zk-SNARK proof, so we include a constant verification cost.
            verification: 1000,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for Delegate {
    fn gas_cost(&self) -> Gas {
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
}

impl GasCost for Undelegate {
    fn gas_cost(&self) -> Gas {
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
}

impl GasCost for UndelegateClaim {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For an UndelegateClaim, nothing is added to the compact block directly. The associated [`Action::Output`]
            // actions will add their costs, but there's nothing to add here.
            compact_block_space: std::mem::size_of::<Nullifier>() as u64,
            // Includes a zk-SNARK proof, so we include a constant verification cost.
            verification: 1000,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for Swap {
    fn gas_cost(&self) -> Gas {
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
            compact_block_space: (std::mem::size_of::<StatePayload>()
                + std::mem::size_of::<BatchSwapOutputData>())
                as u64,
            // Includes a zk-SNARK proof, so we include a constant verification cost.
            verification: 1000,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for SwapClaim {
    fn gas_cost(&self) -> Gas {
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
}

impl GasCost for PositionRewardClaim {
    fn gas_cost(&self) -> Gas {
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

impl GasCost for DaoDeposit {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a DaoDeposit the compact block is not modified.
            compact_block_space: 0u64,
            // Does not include a zk-SNARK proof, so there's no verification cost.
            verification: 0,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for DaoSpend {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a DaoSpend the compact block is not modified.
            compact_block_space: 0u64,
            // Does not include a zk-SNARK proof, so there's no verification cost.
            verification: 0,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for DaoOutput {
    fn gas_cost(&self) -> Gas {
        Gas {
            // Each [`Action`] has a `0` `block_space` cost, since the [`Transaction`] itself
            // will use the encoded size of the complete transaction to calculate the block space.
            block_space: 0,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a DaoOutput this is the byte size of a [`StatePayload`].
            compact_block_space: std::mem::size_of::<StatePayload>() as u64,
            // Does not include a zk-SNARK proof, so there's no verification cost.
            verification: 0,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for IbcAction {
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
                IbcAction::RecvPacket(m) => std::mem::size_of::<StatePayload>() as u64,
                _ => 0u64,
            },
            // Includes a proof in the execution for RecvPacket (TODO: check the other variants).
            verification: match self {
                IbcAction::RecvPacket(m) => 1000 as u64,
                _ => 0u64,
            },
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for ValidatorDefinition {
    fn gas_cost(&self) -> Gas {
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
}

/// Expresses the price of each unit of gas in terms of the staking token.
#[derive(Clone, Debug)]
pub struct GasPrices {
    pub block_space_price: u64,
    pub compact_block_space_price: u64,
    pub verification_price: u64,
    pub execution_price: u64,
}

impl GasPrices {
    pub fn price(&self, gas: &Gas) -> Amount {
        Amount::from(
            self.block_space_price * gas.block_space
                + self.compact_block_space_price * gas.compact_block_space
                + self.verification_price * gas.verification
                + self.execution_price * gas.execution,
        )
    }
}

impl TypeUrl for GasPrices {
    const TYPE_URL: &'static str = "/penumbra.core.transaction.v1alpha1.GasPrices";
}

impl DomainType for GasPrices {
    type Proto = pb::GasPrices;
}

impl From<GasPrices> for pb::GasPrices {
    fn from(prices: GasPrices) -> Self {
        pb::GasPrices {
            block_space_price: prices.block_space_price,
            compact_block_space_price: prices.compact_block_space_price,
            verification_price: prices.verification_price,
            execution_price: prices.execution_price,
        }
    }
}

impl TryFrom<pb::GasPrices> for GasPrices {
    type Error = anyhow::Error;

    fn try_from(proto: pb::GasPrices) -> Result<Self, Self::Error> {
        Ok(GasPrices {
            block_space_price: proto.block_space_price,
            compact_block_space_price: proto.compact_block_space_price,
            verification_price: proto.verification_price,
            execution_price: proto.execution_price,
        })
    }
}
