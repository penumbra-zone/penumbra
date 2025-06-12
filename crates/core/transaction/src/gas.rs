use penumbra_sdk_auction::auction::dutch::actions::{
    ActionDutchAuctionEnd, ActionDutchAuctionSchedule, ActionDutchAuctionWithdraw,
};
use penumbra_sdk_community_pool::{CommunityPoolDeposit, CommunityPoolOutput, CommunityPoolSpend};
use penumbra_sdk_dex::{
    lp::plan::PositionOpenPlan, PositionClose, PositionOpen, PositionWithdraw, Swap, SwapClaim,
};
use penumbra_sdk_fee::Gas;
use penumbra_sdk_funding::liquidity_tournament::{
    ActionLiquidityTournamentVote, LIQUIDITY_TOURNAMENT_VOTE_DENOM_MAX_BYTES,
};
use penumbra_sdk_ibc::IbcRelay;
use penumbra_sdk_keys::symmetric::ENCRYPTED_POSITION_METADATA_SIZE_BYTES;
use penumbra_sdk_shielded_pool::{Ics20Withdrawal, Output, Spend};
use penumbra_sdk_stake::{
    validator::Definition as ValidatorDefinition, Delegate, Undelegate, UndelegateClaim,
};

use penumbra_sdk_governance::{
    DelegatorVote, ProposalDepositClaim, ProposalSubmit, ProposalWithdraw, ValidatorVote,
};

use crate::{
    plan::{ActionPlan, TransactionPlan},
    Action, Transaction,
};

use penumbra_sdk_proto::DomainType;

const NULLIFIER_SIZE: u64 = 2 + 32;
const NOTEPAYLOAD_SIZE: u64 = 32 + 32 + 176;
const SWAPPAYLOAD_SIZE: u64 = 32 + 272;
const ZKPROOF_SIZE: u64 = 192;
// This is an approximation, the actual size is variable
const BSOD_SIZE: u64 = 16 + 16 + 0 + 4 + 64 + 4;

/// Allows [`Action`]s and [`Transaction`]s to statically indicate their relative resource consumption.
/// Since the gas cost needs to be multiplied by a price, the values returned
/// only need to be scaled relatively to each other.
pub trait GasCost {
    fn gas_cost(&self) -> Gas;
}

// Where block space costs are hard-coded instead of calculated in the following functions, the values are based on the approximate byte size of the
// encoded action and ignore the protobuf framing overhead, because it makes only a small difference and simplifies accounting.

pub fn spend_gas_cost() -> Gas {
    Gas {
        // penumbra.core.asset.v1.BalanceCommitment `balance_commitment`  = 32 bytes
        // penumbra.core.component.sct.v1.Nullifier `nullifier`           = 32 bytes
        // penumbra.crypto.decaf377_rdsa.v1.SpendVerificationKey `rk`     = 32 bytes
        // penumbra.crypto.decaf377_rdsa.v1.SpendAuthSignature `auth_sig` = 64 bytes
        // ZKSpendProof `proof`                                           = 192 bytes

        // The block space measured as the byte length of the encoded action.
        block_space: 160 + ZKPROOF_SIZE, // 352 bytes
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block. For a `Spend`, this is the byte size of a `Nullifier`.
        compact_block_space: NULLIFIER_SIZE,
        // Includes a zk-SNARK proof, so we include a constant verification cost.
        verification: 1000,
        // Execution cost is currently hardcoded at 10 for all [`Action`] variants.
        execution: 10,
    }
}

pub fn output_gas_cost() -> Gas {
    Gas {
        // NotePayload `note_payload` = 32 + 32 + 176                    = 240 bytes
        // penumbra.core.asset.v1.BalanceCommitment `balance_commitment` = 32 bytes
        // wrapped_memo_key `wrapped_memo_key`                           = 48 bytes
        // ovk_wrapped_key `ovk_wrapped_key`                             = 48 bytes
        // ZKOutputProof `proof`                                         = 192 bytes

        // The block space measured as the byte length of the encoded action.
        block_space: 128 + NOTEPAYLOAD_SIZE + ZKPROOF_SIZE, // 560 bytes
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        compact_block_space: NOTEPAYLOAD_SIZE,
        // Includes a zk-SNARK proof, so we include a constant verification cost.
        verification: 1000,
        // Execution cost is currently hardcoded at 10 for all [`Action`] variants.
        execution: 10,
    }
}

fn delegate_gas_cost(delegate: &Delegate) -> Gas {
    Gas {
        // The block space measured as the byte length of the encoded action.
        block_space: delegate.encode_to_vec().len() as u64,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a Delegate, nothing is added to the compact block directly. The associated [`Action::Spend`]
        // actions will add their costs, but there's nothing to add here.
        compact_block_space: 0,
        // Does not include a zk-SNARK proof, so there's no verification cost.
        verification: 0,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn undelegate_gas_cost(undelegate: &Undelegate) -> Gas {
    Gas {
        // The block space measured as the byte length of the encoded action.
        block_space: undelegate.encode_to_vec().len() as u64,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For an Undelegate, nothing is added to the compact block directly. The associated [`Action::Spend`]
        // actions will add their costs, but there's nothing to add here.
        compact_block_space: 0,
        // Does not include a zk-SNARK proof, so there's no verification cost.
        verification: 0,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn undelegate_claim_gas_cost() -> Gas {
    Gas {
        // penumbra.core.keys.v1.IdentityKey `validator_identity`         = 32 bytes
        // uint64 `start_epoch_index`                                     = 8 bytes
        // Penalty `penalty`                                              = 32 bytes
        // penumbra.core.asset.v1.BalanceCommitment `balance_commitment`  = 32 bytes
        // uint64 `unbonding_start_height`                                = 8 bytes
        // ZKSpendProof `proof`                                           = 192 bytes

        // The block space measured as the byte length of the encoded action.
        block_space: 112 + ZKPROOF_SIZE, // 304 bytes
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For an `UndelegateClaim`, nothing is added to the compact block directly. The associated [`Action::Output`]
        // actions will add their costs, but there's nothing to add here.
        compact_block_space: 0,
        // Includes a zk-SNARK proof, so we include a constant verification cost.
        verification: 1000,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn validator_definition_gas_cost(validator_definition: &ValidatorDefinition) -> Gas {
    Gas {
        // The block space measured as the byte length of the encoded action.
        block_space: validator_definition.encode_to_vec().len() as u64,
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a ValidatorDefinition the compact block is not modified.
        compact_block_space: 0,
        // Includes a signature verification, so we include a small constant verification cost.
        verification: 200,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn swap_gas_cost() -> Gas {
    Gas {
        // TradingPair `trading_pair`                                = 64 bytes
        // penumbra.core.num.v1.Amount `delta_1_i`                   = 16 bytes
        // penumbra.core.num.v1.Amount `delta_2_i`                   = 16 bytes
        // penumbra.core.asset.v1.BalanceCommitment `fee_commitment` = 32 bytes
        // SwapPayload `payload`                                     = 304 bytes
        // ZKSwapProof `proof`                                       = 192 bytes
        // Batch swap output data                                    = 104 bytes

        // The block space measured as the byte length of the encoded action.
        block_space: 128 + ZKPROOF_SIZE + SWAPPAYLOAD_SIZE + BSOD_SIZE, // 728 bytes
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a `Swap` this is the byte size of a [`StatePayload`] and a [`BatchSwapOutputData`].
        // `Swap`s are batched so technically the cost of the `BatchSwapOutputData` is shared across
        // multiple swaps, but if only one `swap` for a trading pair is performed in a block, that
        // `swap` will add a `BatchSwapOutputData` all on its own.
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
        // penumbra.core.component.sct.v1.Nullifier `nullifier`           = 32 bytes
        // penumbra.core.component.fee.v1.Fee `fee``                      = 48 bytes
        // penumbra.crypto.tct.v1.StateCommitment `output_1_commitment``  = 32 bytes
        // penumbra.crypto.tct.v1.StateCommitment `output_2_commitment`   = 32 bytes
        // BatchSwapOutputData `output_data`                              = 176 bytes
        // uint64 `epoch_duration`                                        = 8 bytes
        // ZKSwapClaimProof `proof`                                       = 192 bytes
        // Batch swap output data                                          = 104 bytes

        // The block space measured as the byte length of the encoded action.
        block_space: 328 + ZKPROOF_SIZE + BSOD_SIZE, // 624 bytes
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a `SwapClaim`, nothing is added to the compact block directly. The associated [`Action::Spend`]
        // and [`Action::Output`] actions will add their costs, but there's nothing to add here.
        compact_block_space: 0,
        // Includes a zk-SNARK proof, so we include a constant verification cost.
        verification: 1000,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn delegator_vote_gas_cost() -> Gas {
    Gas {
        // uint64 `proposal`                                                = 8 bytes
        // uint64 `start_position`                                          = 8 bytes
        // Vote `vote`                                                      = 1 byte
        // penumbra.core.asset.v1.Value `value`                             = 48 bytes
        // penumbra.core.num.v1.Amount `unbonded_amount`                    = 16 bytes
        // penumbra.core.component.sct.v1.Nullifier `nullifier`             = 32 bytes
        // penumbra.crypto.decaf377_rdsa.v1.SpendVerificationKey `rk`       = 32 bytes
        // penumbra.crypto.decaf377_rdsa.v1.SpendAuthSignature `auth_sig`   = 64 bytes
        // ZKDelegatorVoteProof `proof`                                     = 192 bytes

        // The block space measured as the byte length of the encoded action.
        block_space: 209 + ZKPROOF_SIZE, // 401 bytes
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a DelegatorVote the compact block is not modified.
        compact_block_space: 0,
        // Includes a zk-SNARK proof, so we include a constant verification cost.
        verification: 1000,
        // Execution cost is currently hardcoded at 10 for all Action variants.
        execution: 10,
    }
}

fn position_withdraw_gas_cost() -> Gas {
    Gas {
        // PositionId `position_id`                                        = 32 bytes
        // penumbra.core.asset.v1.BalanceCommitment `reserves_commitment`  = 32 bytes
        // uint64 `sequence`                                               = 8 bytes

        // The block space measured as the byte length of the encoded action.
        block_space: 72, // 72 bytes
        // The compact block space cost is based on the byte size of the data the [`Action`] adds
        // to the compact block.
        // For a PositionWithdraw the compact block is not modified.
        compact_block_space: 0,
        // Does not include a zk-SNARK proof, so there's no verification cost.
        verification: 0,
        // Execution cost is currently hardcoded at 10 for all `Action`` variants.
        // Reminder: Any change to this execution gas vector must also be reflected
        // in updates to the dutch auction gas vectors.
        execution: 10,
    }
}

fn dutch_auction_schedule_gas_cost(dutch_action_schedule: &ActionDutchAuctionSchedule) -> Gas {
    Gas {
        // penumbra.core.asset.v1.Value `input` = 48 bytes
        // penumbra.core.asset.v1.AssetId `output_id` = 32 bytes
        // penumbra.core.num.v1.Amount `max_output` = 16 bytes
        // penumbra.core.num.v1.Amount `min_output` = 16 bytes
        // uint64 `start_height` = 8 bytes
        // uint64 `end_height` = 8 bytes
        // uint64 `step_count` = 8 bytes
        // bytes `nonce` = 32 bytes
        block_space: 168,
        compact_block_space: 0,
        verification: 50,
        // Currently, we make the execution cost for DA actions proportional to the number of steps
        // and costs of position open/close in dutch action. The gas cost is calculated by:
        // 2 * step_count * (`PositionOpen`` + `PositionClose` cost).
        execution: 2 * dutch_action_schedule.description.step_count * (10 + 10),
    }
}

fn dutch_auction_end_gas_cost() -> Gas {
    Gas {
        // AuctionId `auction_id` = 32 bytes
        block_space: 32, // 32 bytes
        compact_block_space: 0,
        verification: 0,
        execution: 10,
    }
}

fn dutch_auction_withdraw_gas_cost() -> Gas {
    Gas {
        // AuctionId `auction_id` = 32 bytes
        // uint64 `seq`= 8 bytes
        // penumbra.core.asset.v1.BalanceCommitment `reserves_commitment` = 32 bytes
        block_space: 72, // 72 bytes
        compact_block_space: 0,
        verification: 0,
        execution: 10,
    }
}

fn liquidity_tournament_vote_gas_cost() -> Gas {
    Gas {
        block_space:
        // LiquidityTournamentVoteBody body = 1;
        (
            // asset.v1.Denom incentivized = 1; (restricted to MAX bytes)
            LIQUIDITY_TOURNAMENT_VOTE_DENOM_MAX_BYTES as u64
            // keys.v1.Address rewards_recipient = 2; (the larger of the two exclusive fields)
            + 223
            // uint64 start_position = 3;
            + 8
            // asset.v1.Value value = 4;
            + 48
            // sct.v1.Nullifier nullifier = 5;
            + 32
            // crypto.decaf377_rdsa.v1.SpendVerificationKey rk = 6;
            + 32
        )
        // crypto.decaf377_rdsa.v1.SpendAuthSignature auth_sig = 2;
        + 64
        // ZKLiquidityTournamentVoteProof proof = 3;
        + ZKPROOF_SIZE,
        // Each vote will, pessimistically, create one output for the reward.
        compact_block_space: NOTEPAYLOAD_SIZE,
        verification: 1000,
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
            ActionPlan::UndelegateClaim(_) => undelegate_claim_gas_cost(),
            ActionPlan::Swap(_) => swap_gas_cost(),
            ActionPlan::SwapClaim(_) => swap_claim_gas_cost(),
            ActionPlan::DelegatorVote(_) => delegator_vote_gas_cost(),
            ActionPlan::PositionWithdraw(_) => position_withdraw_gas_cost(),
            ActionPlan::ActionDutchAuctionSchedule(das) => das.gas_cost(),
            ActionPlan::ActionDutchAuctionEnd(_) => dutch_auction_end_gas_cost(),
            ActionPlan::ActionDutchAuctionWithdraw(_) => dutch_auction_withdraw_gas_cost(),

            ActionPlan::Delegate(d) => d.gas_cost(),
            ActionPlan::Undelegate(u) => u.gas_cost(),
            ActionPlan::ValidatorDefinition(vd) => vd.gas_cost(),
            ActionPlan::IbcAction(i) => i.gas_cost(),
            ActionPlan::ProposalSubmit(ps) => ps.gas_cost(),
            ActionPlan::ProposalWithdraw(pw) => pw.gas_cost(),
            ActionPlan::ValidatorVote(v) => v.gas_cost(),
            ActionPlan::ProposalDepositClaim(pdc) => pdc.gas_cost(),
            ActionPlan::PositionOpen(po) => po.gas_cost(),
            ActionPlan::PositionClose(pc) => pc.gas_cost(),
            ActionPlan::CommunityPoolSpend(ds) => ds.gas_cost(),
            ActionPlan::CommunityPoolOutput(d) => d.gas_cost(),
            ActionPlan::CommunityPoolDeposit(dd) => dd.gas_cost(),
            ActionPlan::Ics20Withdrawal(w) => w.gas_cost(),
            ActionPlan::ActionLiquidityTournamentVote(_) => liquidity_tournament_vote_gas_cost(),
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
            Action::Ics20Withdrawal(withdrawal) => withdrawal.gas_cost(),
            Action::CommunityPoolDeposit(deposit) => deposit.gas_cost(),
            Action::CommunityPoolSpend(spend) => spend.gas_cost(),
            Action::CommunityPoolOutput(output) => output.gas_cost(),
            Action::IbcRelay(x) => x.gas_cost(),
            Action::ValidatorDefinition(x) => x.gas_cost(),
            Action::ActionDutchAuctionSchedule(action_dutch_auction_schedule) => {
                action_dutch_auction_schedule.gas_cost()
            }
            Action::ActionDutchAuctionEnd(action_dutch_auction_end) => {
                action_dutch_auction_end.gas_cost()
            }
            Action::ActionDutchAuctionWithdraw(action_dutch_auction_withdraw) => {
                action_dutch_auction_withdraw.gas_cost()
            }
            Action::ActionLiquidityTournamentVote(action_liquidity_tournament_vote) => {
                action_liquidity_tournament_vote.gas_cost()
            }
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
        delegate_gas_cost(&self)
    }
}

impl GasCost for Undelegate {
    fn gas_cost(&self) -> Gas {
        undelegate_gas_cost(&self)
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
            // The block space measured as the byte length of the encoded action.
            block_space: self.encode_to_vec().len() as u64,
            // In the case of a proposal submission, the compact block cost is zero.
            // The compact block is only modified it the proposal is ratified.
            // And when that's the case, the cost is mutualized.
            compact_block_space: 0,
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
            // The block space measured as the byte length of the encoded action.
            block_space: self.encode_to_vec().len() as u64,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a ProposalWithdraw the compact block is not modified.
            compact_block_space: 0,
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
            // The block space measured as the byte length of the encoded action.
            block_space: self.encode_to_vec().len() as u64,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a ValidatorVote the compact block is not modified.
            compact_block_space: 0,
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
            // The block space measured as the byte length of the encoded action.
            block_space: self.encode_to_vec().len() as u64,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a ProposalDepositClaim the compact block is not modified.
            compact_block_space: 0,
            // Does not include a zk-SNARK proof, so there's no verification cost.
            verification: 0,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for PositionOpenPlan {
    fn gas_cost(&self) -> Gas {
        let padding = if self.metadata.is_none() {
            ENCRYPTED_POSITION_METADATA_SIZE_BYTES as u64
        } else {
            0
        };
        Gas {
            // The block space measured as the byte length of the encoded action.
            // But, we also add padding to not penalize including a ciphertext.
            block_space: self.position.encode_to_vec().len() as u64 + padding,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a PositionOpen the compact block is not modified.
            compact_block_space: 0,
            // There are some small validations performed so a token amount of gas is charged.
            verification: 50,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            // Reminder: Any change to this execution gas vector must also be reflected
            // in updates to the dutch auction gas vectors.
            execution: 10,
        }
    }
}

impl GasCost for PositionOpen {
    fn gas_cost(&self) -> Gas {
        Gas {
            // The block space measured as the byte length of the encoded action.
            block_space: self.encode_to_vec().len() as u64,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a PositionOpen the compact block is not modified.
            compact_block_space: 0,
            // There are some small validations performed so a token amount of gas is charged.
            verification: 50,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            // Reminder: Any change to this execution gas vector must also be reflected
            // in updates to the dutch auction gas vectors.
            execution: 10,
        }
    }
}

impl GasCost for PositionClose {
    fn gas_cost(&self) -> Gas {
        Gas {
            // The block space measured as the byte length of the encoded action.
            block_space: self.encode_to_vec().len() as u64,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a PositionClose the compact block is not modified.
            compact_block_space: 0,
            // Does not include a zk-SNARK proof, so there's no verification cost.
            verification: 0,
            // Execution cost is currently hardcoded at 10 for all Action variants.
            // Reminder: Any change to this execution gas vector must also be reflected
            // in updates to the dutch auction gas vectors.
            execution: 10,
        }
    }
}

impl GasCost for PositionWithdraw {
    fn gas_cost(&self) -> Gas {
        position_withdraw_gas_cost()
    }
}

impl GasCost for Ics20Withdrawal {
    fn gas_cost(&self) -> Gas {
        Gas {
            // The block space measured as the byte length of the encoded action.
            block_space: self.encode_to_vec().len() as u64,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a Ics20Withdrawal the compact block is not modified.
            compact_block_space: 0,
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
            // The block space measured as the byte length of the encoded action.
            block_space: self.encode_to_vec().len() as u64,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a CommunityPoolDeposit the compact block is not modified.
            compact_block_space: 0,
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
            // The block space measured as the byte length of the encoded action.
            block_space: self.encode_to_vec().len() as u64,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a CommunityPoolSpend the compact block is not modified.
            compact_block_space: 0,
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
            // The block space measured as the byte length of the encoded action.
            block_space: self.encode_to_vec().len() as u64,
            // The compact block space cost is based on the byte size of the data the [`Action`] adds
            // to the compact block.
            // For a IbcAction this is the byte size of a [`StatePayload`].
            compact_block_space: match self {
                // RecvPacket will mint a note if successful.
                IbcRelay::RecvPacket(_) => NOTEPAYLOAD_SIZE,
                _ => 0,
            },
            // Includes a proof in the execution for RecvPacket (TODO: check the other variants).
            verification: match self {
                IbcRelay::RecvPacket(_) => 1000,
                _ => 0,
            },
            // Execution cost is currently hardcoded at 10 for all Action variants.
            execution: 10,
        }
    }
}

impl GasCost for ValidatorDefinition {
    fn gas_cost(&self) -> Gas {
        validator_definition_gas_cost(&self)
    }
}

impl GasCost for ActionDutchAuctionSchedule {
    fn gas_cost(&self) -> Gas {
        dutch_auction_schedule_gas_cost(&self)
    }
}

impl GasCost for ActionDutchAuctionEnd {
    fn gas_cost(&self) -> Gas {
        dutch_auction_end_gas_cost()
    }
}

impl GasCost for ActionDutchAuctionWithdraw {
    fn gas_cost(&self) -> Gas {
        dutch_auction_withdraw_gas_cost()
    }
}

impl GasCost for ActionLiquidityTournamentVote {
    fn gas_cost(&self) -> Gas {
        liquidity_tournament_vote_gas_cost()
    }
}
