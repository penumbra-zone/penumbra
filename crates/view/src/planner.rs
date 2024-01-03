use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Formatter},
    mem,
};

use anyhow::Result;
use rand::{CryptoRng, RngCore};
use tracing::instrument;

use penumbra_asset::{asset, Balance, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_chain::params::{ChainParameters, FmdParameters};
use penumbra_community_pool::CommunityPoolDeposit;
use penumbra_dex::{
    lp::action::{PositionClose, PositionOpen},
    lp::plan::PositionWithdrawPlan,
    lp::position::{self, Position},
    lp::Reserves,
    swap::SwapPlaintext,
    swap::SwapPlan,
    swap_claim::SwapClaimPlan,
    TradingPair,
};
use penumbra_fee::{Fee, GasPrices};
use penumbra_governance::{
    proposal_state, DelegatorVotePlan, Proposal, ProposalDepositClaim, ProposalSubmit,
    ProposalWithdraw, ValidatorVote, Vote,
};
use penumbra_ibc::IbcRelay;
use penumbra_keys::{keys::AddressIndex, Address};
use penumbra_num::Amount;
use penumbra_proto::view::v1alpha1::{NotesForVotingRequest, NotesRequest};
use penumbra_shielded_pool::{Ics20Withdrawal, Note, OutputPlan, SpendPlan};
use penumbra_stake::{rate::RateData, validator};
use penumbra_stake::{IdentityKey, UndelegateClaimPlan};
use penumbra_tct as tct;
use penumbra_transaction::{
    gas::GasCost,
    memo::MemoPlaintext,
    plan::{ActionPlan, MemoPlan, TransactionPlan},
};

use crate::{SpendableNoteRecord, ViewClient};

/// A planner for a [`TransactionPlan`] that can fill in the required spends and change outputs upon
/// finalization to make a transaction balance.
pub struct Planner<R: RngCore + CryptoRng> {
    rng: R,
    balance: Balance,
    vote_intents: BTreeMap<u64, VoteIntent>,
    plan: TransactionPlan,
    ibc_actions: Vec<IbcRelay>,
    gas_prices: GasPrices,
    // IMPORTANT: if you add more fields here, make sure to clear them when the planner is finished
}

#[derive(Debug, Clone)]
struct VoteIntent {
    start_block_height: u64,
    start_position: tct::Position,
    rate_data: BTreeMap<IdentityKey, RateData>,
    vote: Vote,
}

impl<R: RngCore + CryptoRng> Debug for Planner<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("balance", &self.balance)
            .field("plan", &self.plan)
            .finish()
    }
}

impl<R: RngCore + CryptoRng> Planner<R> {
    /// Create a new planner.
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            balance: Balance::default(),
            vote_intents: BTreeMap::default(),
            plan: TransactionPlan::default(),
            ibc_actions: Vec::new(),
            gas_prices: GasPrices::zero(),
        }
    }

    /// Set the current gas prices for fee prediction.
    #[instrument(skip(self))]
    pub fn set_gas_prices(&mut self, gas_prices: GasPrices) -> &mut Self {
        self.gas_prices = gas_prices;
        self
    }

    /// Get the current transaction balance of the planner.
    pub fn balance(&self) -> &Balance {
        &self.balance
    }

    /// Get all the note requests necessary to fulfill the current [`Balance`].
    pub fn notes_requests(
        &self,
        source: AddressIndex,
    ) -> (Vec<NotesRequest>, Vec<NotesForVotingRequest>) {
        (
            self.balance
                .required()
                .map(|Value { asset_id, amount }| NotesRequest {
                    asset_id: Some(asset_id.into()),
                    address_index: Some(source.into()),
                    amount_to_spend: Some(amount.into()),
                    include_spent: false,
                })
                .collect(),
            self.vote_intents
                .iter()
                .map(
                    |(
                        _proposal, // The request only cares about the start block height
                        VoteIntent {
                            start_block_height, ..
                        },
                    )| NotesForVotingRequest {
                        votable_at_height: *start_block_height,
                        address_index: Some(source.into()),
                    },
                )
                .collect(),
        )
    }

    /// Set the expiry height for the transaction plan.
    #[instrument(skip(self))]
    pub fn expiry_height(&mut self, expiry_height: u64) -> &mut Self {
        self.plan.transaction_parameters.expiry_height = expiry_height;
        self
    }

    /// Set a memo for this transaction plan.
    ///
    /// Errors if the memo is too long.
    #[instrument(skip(self))]
    pub fn memo(&mut self, memo: MemoPlaintext) -> anyhow::Result<&mut Self> {
        self.plan.memo = Some(MemoPlan::new(&mut self.rng, memo)?);
        Ok(self)
    }

    /// Add a fee to the transaction plan.
    ///
    /// This function should be called once.
    #[instrument(skip(self))]
    pub fn fee(&mut self, fee: Fee) -> &mut Self {
        self.balance += fee.0;
        self.plan.transaction_parameters.fee = fee;
        self
    }

    /// Calculate gas cost-based fees and add to the transaction plan.
    ///
    /// This function should be called once.
    // TODO: clarify why we have both `add_gas_fees` and `fee`
    // should one be `auto_fee` and the other `set_fee`?
    #[instrument(skip(self))]
    pub fn add_gas_fees(&mut self) -> &mut Self {
        let minimum_fee = self.gas_prices.price(&self.plan.gas_cost());

        // Since paying the fee possibly requires adding an additional Spend to the
        // transaction, which would then change the fee calculation, we multiply the
        // fee here by a factor of 2 and then recalculate and capture the excess as
        // change outputs.
        let fee = Fee::from_staking_token_amount(minimum_fee * Amount::from(2u32));
        self.balance += fee.0;
        self.plan.transaction_parameters.fee = fee;
        self
    }

    /// Spend a specific positioned note in the transaction.
    ///
    /// If you don't use this method to specify spends, they will be filled in automatically from
    /// the view service when the plan is [`finish`](Planner::finish)ed.
    #[instrument(skip(self))]
    pub fn spend(&mut self, note: Note, position: tct::Position) -> &mut Self {
        let spend = SpendPlan::new(&mut self.rng, note, position).into();
        self.action(spend);
        self
    }

    /// Open a liquidity position in the order book.
    #[instrument(skip(self))]
    pub fn position_open(&mut self, position: Position) -> &mut Self {
        self.action(ActionPlan::PositionOpen(PositionOpen { position }));
        self
    }

    /// Close a liquidity position in the order book.
    #[instrument(skip(self))]
    pub fn position_close(&mut self, position_id: position::Id) -> &mut Self {
        self.action(ActionPlan::PositionClose(PositionClose { position_id }));
        self
    }

    /// Withdraw a liquidity position in the order book.
    #[instrument(skip(self))]
    pub fn position_withdraw(
        &mut self,
        position_id: position::Id,
        reserves: Reserves,
        pair: TradingPair,
    ) -> &mut Self {
        self.action(ActionPlan::PositionWithdraw(PositionWithdrawPlan::new(
            reserves,
            position_id,
            pair,
        )));
        self
    }

    /// Perform a swap claim based on an input swap NFT with a pre-paid fee.
    #[instrument(skip(self))]
    pub fn swap_claim(&mut self, plan: SwapClaimPlan) -> &mut Self {
        // Nothing needs to be spent, since the fee is pre-paid and the
        // swap NFT will be automatically consumed when the SwapClaim action
        // is processed by the validators.
        // TODO: need to set the intended fee so the tx actually balances,
        // otherwise the planner will create an output
        self.action(plan.into());
        self
    }

    /// Perform a swap based on input notes in the transaction.
    #[instrument(skip(self))]
    pub fn swap(
        &mut self,
        input_value: Value,
        into_asset: asset::Id,
        swap_claim_fee: Fee,
        claim_address: Address,
    ) -> Result<&mut Self> {
        // Determine the canonical order for the assets being swapped.
        // This will determine whether the input amount is assigned to delta_1 or delta_2.
        let trading_pair = TradingPair::new(input_value.asset_id, into_asset);

        // If `trading_pair.asset_1` is the input asset, then `delta_1` is the input amount,
        // and `delta_2` is 0.
        //
        // Otherwise, `delta_1` is 0, and `delta_2` is the input amount.
        let (delta_1, delta_2) = if trading_pair.asset_1() == input_value.asset_id {
            (input_value.amount, 0u64.into())
        } else {
            (0u64.into(), input_value.amount)
        };

        // If there is no input, then there is no swap.
        if delta_1 == Amount::zero() && delta_2 == Amount::zero() {
            anyhow::bail!("No input value for swap");
        }

        // Create the `SwapPlaintext` representing the swap to be performed:
        let swap_plaintext = SwapPlaintext::new(
            &mut self.rng,
            trading_pair,
            delta_1,
            delta_2,
            swap_claim_fee,
            claim_address,
        );

        let swap = SwapPlan::new(&mut self.rng, swap_plaintext).into();
        self.action(swap);

        Ok(self)
    }

    /// Add an output note from this transaction.
    ///
    /// Any unused output value will be redirected back to the originating address as change notes
    /// when the plan is [`finish`](Builder::finish)ed.
    #[instrument(skip(self))]
    pub fn output(&mut self, value: Value, address: Address) -> &mut Self {
        let output = OutputPlan::new(&mut self.rng, value, address).into();
        self.action(output);
        self
    }

    /// Add a delegation to this transaction.
    ///
    /// If you don't specify spends or outputs as well, they will be filled in automatically.
    #[instrument(skip(self))]
    pub fn delegate(&mut self, unbonded_amount: u128, rate_data: RateData) -> &mut Self {
        let delegation = rate_data.build_delegate(unbonded_amount).into();
        self.action(delegation);
        self
    }

    /// Add an undelegation to this transaction.
    ///
    /// TODO: can we put the chain parameters into the planner at the start, so we can compute end_epoch_index?
    #[instrument(skip(self))]
    pub fn undelegate(&mut self, delegation_amount: Amount, rate_data: RateData) -> &mut Self {
        let undelegation = rate_data.build_undelegate(delegation_amount).into();
        self.action(undelegation);
        self
    }

    /// Add an undelegate claim to this transaction.
    #[instrument(skip(self))]
    pub fn undelegate_claim(&mut self, claim_plan: UndelegateClaimPlan) -> &mut Self {
        self.action(ActionPlan::UndelegateClaim(claim_plan));
        self
    }

    /// Upload a validator definition in this transaction.
    #[instrument(skip(self))]
    pub fn validator_definition(&mut self, new_validator: validator::Definition) -> &mut Self {
        self.action(ActionPlan::ValidatorDefinition(new_validator));
        self
    }

    /// Submit a new governance proposal in this transaction.
    #[instrument(skip(self))]
    pub fn proposal_submit(&mut self, proposal: Proposal, deposit_amount: Amount) -> &mut Self {
        self.action(ActionPlan::ProposalSubmit(ProposalSubmit {
            proposal,
            deposit_amount,
        }));
        self
    }

    /// Withdraw a governance proposal in this transaction.
    #[instrument(skip(self))]
    pub fn proposal_withdraw(&mut self, proposal: u64, reason: String) -> &mut Self {
        self.action(ActionPlan::ProposalWithdraw(ProposalWithdraw {
            proposal,
            reason,
        }));
        self
    }

    /// Claim a governance proposal deposit in this transaction.
    #[instrument(skip(self))]
    pub fn proposal_deposit_claim(
        &mut self,
        proposal: u64,
        deposit_amount: Amount,
        outcome: proposal_state::Outcome<()>,
    ) -> &mut Self {
        self.action(ActionPlan::ProposalDepositClaim(ProposalDepositClaim {
            proposal,
            deposit_amount,
            outcome,
        }));
        self
    }

    /// Deposit a value into the Community Pool.
    #[instrument(skip(self))]
    pub fn community_pool_deposit(&mut self, value: Value) -> &mut Self {
        self.action(ActionPlan::CommunityPoolDeposit(CommunityPoolDeposit {
            value,
        }));
        self
    }

    /// Cast a validator vote in this transaction.
    #[instrument(skip(self))]
    pub fn validator_vote(&mut self, vote: ValidatorVote) -> &mut Self {
        self.action(ActionPlan::ValidatorVote(vote));
        self
    }

    /// Perform an ICS-20 withdrawal
    #[instrument(skip(self))]
    pub fn ics20_withdrawal(&mut self, withdrawal: Ics20Withdrawal) -> &mut Self {
        self.action(ActionPlan::Withdrawal(withdrawal));
        self
    }

    /// Perform an IBC action
    #[instrument(skip(self))]
    pub fn ibc_action(&mut self, ibc_action: IbcRelay) -> &mut Self {
        self.action(ActionPlan::IbcAction(ibc_action));
        self
    }

    /// Vote with all possible vote weight on a given proposal.
    ///
    /// Voting twice on the same proposal in the same planner will overwrite the previous vote.
    #[instrument(skip(self, start_position, start_rate_data))]
    pub fn delegator_vote(
        &mut self,
        proposal: u64,
        start_block_height: u64,
        start_position: tct::Position,
        start_rate_data: BTreeMap<IdentityKey, RateData>,
        vote: Vote,
    ) -> &mut Self {
        self.vote_intents.insert(
            proposal,
            VoteIntent {
                start_position,
                start_block_height,
                vote,
                rate_data: start_rate_data,
            },
        );
        self
    }

    /// Vote with a specific positioned note in the transaction.
    ///
    /// If you don't use this method to specify votes, they will be filled in automatically from the
    /// implied voting intent by [`vote`](Planner::vote) when the plan is
    /// [`finish`](Planner::finish)ed.
    #[instrument(skip(self, start_position))]
    pub fn delegator_vote_precise(
        &mut self,
        proposal: u64,
        start_position: tct::Position,
        vote: Vote,
        note: Note,
        position: tct::Position,
        unbonded_amount: Amount,
    ) -> &mut Self {
        let vote = DelegatorVotePlan::new(
            &mut self.rng,
            proposal,
            start_position,
            vote,
            note,
            position,
            unbonded_amount,
        )
        .into();
        self.action(vote);
        self
    }

    fn action(&mut self, action: ActionPlan) -> &mut Self {
        // Track the contribution of the action to the transaction's balance
        self.balance += action.balance();

        // Add the action to the plan
        self.plan.actions.push(action);
        self
    }

    /// Add spends and change outputs as required to balance the transaction, using the view service
    /// provided to supply the notes and other information.
    ///
    /// Clears the contents of the planner, which can be re-used.
    pub async fn plan<V: ViewClient>(
        &mut self,
        view: &mut V,
        source: AddressIndex,
    ) -> anyhow::Result<TransactionPlan> {
        // Gather all the information needed from the view service
        let chain_params = view.app_params().await?.chain_params;
        let fmd_params = view.fmd_parameters().await?;

        // Calculate the gas that needs to be paid for the transaction based on the configured gas prices.
        // Note that _paying the fee might incur an additional `Spend` action_, thus increasing the fee,
        // so we slightly overpay here and then capture the excess as change later during `plan_with_spendable_and_votable_notes`.
        // Add the fee to the planner's internal balance.
        self.add_gas_fees();

        let mut spendable_notes = Vec::new();
        let mut voting_notes = Vec::new();
        let (spendable_requests, voting_requests) = self.notes_requests(source);
        for request in spendable_requests {
            let notes = view.notes(request).await?;
            spendable_notes.extend(notes);
        }
        for request in voting_requests {
            let notes = view.notes_for_voting(request).await?;
            voting_notes.push(notes);
        }

        // Plan the transaction using the gathered information

        let self_address = view.address_by_index(source).await?;
        self.plan_with_spendable_and_votable_notes(
            &chain_params,
            &fmd_params,
            spendable_notes,
            voting_notes,
            self_address,
        )
    }

    /// Add spends and change outputs as required to balance the transaction, using the spendable
    /// notes provided. It is the caller's responsibility to ensure that the notes are the result of
    /// collected responses to the requests generated by an immediately preceding call to
    /// [`Planner::note_requests`].
    ///
    /// Clears the contents of the planner, which can be re-used.
    #[instrument(skip(
        self,
        chain_params,
        fmd_params,
        self_address,
        spendable_notes,
        votable_notes,
    ))]
    pub fn plan_with_spendable_and_votable_notes(
        &mut self,
        chain_params: &ChainParameters,
        fmd_params: &FmdParameters,
        spendable_notes: Vec<SpendableNoteRecord>,
        votable_notes: Vec<Vec<(SpendableNoteRecord, IdentityKey)>>,
        self_address: Address,
    ) -> anyhow::Result<TransactionPlan> {
        tracing::debug!(plan = ?self.plan, balance = ?self.balance, "finalizing transaction");

        // Fill in the chain id based on the view service
        self.plan.transaction_parameters.chain_id = chain_params.chain_id.clone();

        // Add the required spends to the planner
        for record in spendable_notes {
            self.spend(record.note, record.position);
        }
        // Add any IBC actions to the planner
        for ibc_action in self.ibc_actions.clone() {
            self.ibc_action(ibc_action);
        }

        // Add the required votes to the planner
        for (
            records,
            (
                proposal,
                VoteIntent {
                    start_position,
                    vote,
                    rate_data,
                    ..
                },
            ),
        ) in votable_notes
            .into_iter()
            .chain(std::iter::repeat(vec![])) // Chain with infinite repeating no notes, so the zip doesn't stop early
            .zip(mem::take(&mut self.vote_intents).into_iter())
        {
            // Keep track of whether we successfully could vote on this proposal
            let mut voted = false;

            for (record, identity_key) in records {
                // Vote with precisely this note on the proposal, computing the correct exchange
                // rate for self-minted vote receipt tokens using the exchange rate of the validator
                // at voting start time. If the validator was not active at the start of the
                // proposal, the vote will be rejected by stateful verification, so skip the note
                // and continue to the next one.
                let Some(rate_data) = rate_data.get(&identity_key) else {
                    continue;
                };
                let unbonded_amount = rate_data
                    .unbonded_amount(record.note.amount().value())
                    .into();

                // If the delegation token is unspent, "roll it over" by spending it (this will
                // result in change sent back to us). This unlinks nullifiers used for voting on
                // multiple non-overlapping proposals, increasing privacy.
                if record.height_spent.is_none() {
                    self.spend(record.note.clone(), record.position);
                }

                self.delegator_vote_precise(
                    proposal,
                    start_position,
                    vote,
                    record.note,
                    record.position,
                    unbonded_amount,
                );

                voted = true;
            }

            if !voted {
                // If there are no notes to vote with, return an error, because otherwise the user
                // would compose a transaction that would not satisfy their intention, and would
                // silently eat the fee.
                anyhow::bail!(
                    "can't vote on proposal {} because no delegation notes were staked to an active validator when voting started",
                    proposal
                );
            }
        }

        // Since we over-estimate the fees to be paid upfront by a fixed multiple to account
        // for the cost of any additional `Spend` actions necessary to pay the fee, we need
        // to now calculate the transaction's fee again and capture the excess as change
        // by subtracting the excess from the required value balance.
        let tx_real_fee = self.gas_prices.price(&self.plan.gas_cost());
        let excess_fee_spent = self.plan.transaction_parameters.fee.amount() - tx_real_fee;
        self.balance -= Value {
            amount: excess_fee_spent,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        self.plan.transaction_parameters.fee = Fee::from_staking_token_amount(tx_real_fee);

        // For any remaining provided balance, make a single change note for each
        for value in self.balance.provided().collect::<Vec<_>>() {
            self.output(value, self_address);
        }

        // All actions have now been added, so check to make sure that you don't build and submit an
        // empty transaction
        if self.plan.actions.is_empty() {
            anyhow::bail!("planned transaction would be empty, so should not be submitted");
        }

        // Now the transaction should be fully balanced, unless we didn't have enough to spend
        if !self.balance.is_zero() {
            anyhow::bail!(
                "balance is non-zero after attempting to balance transaction: {:?}",
                self.balance
            );
        }

        // If there are outputs, we check that a memo has been added. If not, we add a blank memo.
        if self.plan.num_outputs() > 0 && self.plan.memo.is_none() {
            self.memo(MemoPlaintext::blank_memo(self_address.clone()))
                .expect("empty string is a valid memo");
        } else if self.plan.num_outputs() == 0 && self.plan.memo.is_some() {
            anyhow::bail!("if no outputs, no memo should be added");
        }

        // Add clue plans for `Output`s.
        let precision_bits = fmd_params.precision_bits;
        self.plan
            .populate_detection_data(&mut self.rng, precision_bits.into());

        tracing::debug!(plan = ?self.plan, "finished balancing transaction");

        // Clear the planner and pull out the plan to return
        self.balance = Balance::zero();
        self.vote_intents = BTreeMap::new();
        self.ibc_actions = Vec::new();
        self.gas_prices = GasPrices::zero();
        let plan = mem::take(&mut self.plan);

        Ok(plan)
    }
}
