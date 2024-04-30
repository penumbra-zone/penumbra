use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Formatter},
    mem,
};

use anyhow::anyhow;
use anyhow::Result;
use ark_std::iterable::Iterable;
use penumbra_sct::epoch::Epoch;
use rand::{CryptoRng, RngCore};
use rand_core::OsRng;
use tracing::instrument;

use penumbra_asset::{asset, Balance, Value, STAKING_TOKEN_ASSET_ID, STAKING_TOKEN_DENOM};
// use penumbra_auction::auction::dutch::actions::ActionDutchAuctionWithdrawPlan;
// use penumbra_auction::auction::dutch::DutchAuctionDescription;
// use penumbra_auction::auction::{
//     dutch::actions::{ActionDutchAuctionEnd, ActionDutchAuctionSchedule},
//     AuctionId,
// };
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
use penumbra_fee::{Fee, FeeTier, Gas, GasPrices};
use penumbra_governance::{
    proposal_state, DelegatorVotePlan, Proposal, ProposalDepositClaim, ProposalSubmit,
    ProposalWithdraw, ValidatorVote, Vote,
};
use penumbra_ibc::IbcRelay;
use penumbra_keys::{keys::AddressIndex, Address};
use penumbra_num::Amount;
use penumbra_proto::view::v1::{NotesForVotingRequest, NotesRequest};
use penumbra_shielded_pool::{Ics20Withdrawal, Note, OutputPlan, SpendPlan};
use penumbra_stake::{rate::RateData, validator, IdentityKey, UndelegateClaimPlan};
use penumbra_tct as tct;
use penumbra_transaction::{
    gas::GasCost,
    memo::MemoPlaintext,
    plan::{ActionPlan, MemoPlan, TransactionPlan},
    TransactionParameters,
};

use crate::{SpendableNoteRecord, ViewClient};

/// A planner for a [`TransactionPlan`] that can fill in the required spends and change outputs upon
/// finalization to make a transaction balance.
pub struct Planner<R: RngCore + CryptoRng> {
    rng: R,
    balance: Balance,
    vote_intents: BTreeMap<u64, VoteIntent>,
    plan: TransactionPlan,
    gas_prices: GasPrices,
    fee_tier: FeeTier,
    actions: Vec<ActionPlan>,
    change_outputs: BTreeMap<asset::Id, OutputPlan>,
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
    /// Creates a new `Planner` instance with default settings.
    /// The planner is used to assemble and manage transaction plans, incorporating
    /// various actions like spending and receiving, as well as handling gas and fees.
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            balance: Balance::default(),
            vote_intents: BTreeMap::default(),
            plan: TransactionPlan::default(),
            gas_prices: GasPrices::zero(),
            fee_tier: FeeTier::default(),
            actions: Vec::new(),
            change_outputs: BTreeMap::new(),
        }
    }

    /// Calculates the total balance by summing up the balance of all actions and change outputs.
    fn calculate_balance(&self) -> Balance {
        let mut balance = Balance::zero();
        for action in &self.actions {
            balance += action.balance();
        }
        for action in self.change_outputs.values() {
            balance += action.balance();
        }

        balance
    }

    /// Calculates the balance after accounting for the base fee estimation.
    /// This helps understand the net balance available after fees are applied.
    fn calculate_balance_with_fees(&self, base_fee_estimation: Fee) -> Balance {
        self.calculate_balance() - base_fee_estimation.0
    }

    /// Adds an action plan to the list of actions within the planner.
    /// This is used when assembling the components of a transaction.
    fn push(&mut self, action: ActionPlan) {
        self.actions.push(action);
    }

    /// Estimates the total gas usage of the transaction based on all actions and change outputs.
    fn gas_estimate(&self) -> Gas {
        let mut gas = Gas::zero();
        for action in &self.actions {
            gas += action.gas_cost();
        }
        for action in self.change_outputs.values() {
            gas += ActionPlan::from(action.clone()).gas_cost();
        }

        gas
    }

    /// Estimates the total fees for the transaction based on the estimated gas usage
    /// and the current gas prices and fee tier.
    fn fee_estimate(&self, gas_prices: &GasPrices, fee_tier: &FeeTier) -> Fee {
        let base_fee: Fee = Fee::from_staking_token_amount(gas_prices.fee(&self.gas_estimate()));

        base_fee.apply_tier(*fee_tier)
    }

    /// Refreshes the change outputs based on the current balance and specified change address.
    /// This creates new change outputs for any excess value after actions are accounted for.
    fn refresh_change(&mut self, change_address: Address) {
        println!("entered refresh_change!");
        println!("refresh change before: {:?}", self.change_outputs);

        self.change_outputs = BTreeMap::new();
        // For each "provided" balance component, create a change note.
        for value in self.calculate_balance().provided() {
            self.change_outputs.insert(
                value.asset_id,
                OutputPlan::new(&mut OsRng, value, change_address.clone()),
            );
        }

        println!("refresh change after: {:?}", self.change_outputs);
    }

    /// Adjusts the change outputs to account for transaction fees.
    /// This reduces the change amount by the estimated fee to ensure the transaction
    /// balances correctly after fees are considered.
    fn adjust_change_for_fee(&mut self, fee: Fee) {
        println!("entered adjust_change_for_fee!");

        if !(self.change_outputs.is_empty()) {
            self.change_outputs.entry(fee.0.asset_id).and_modify(|e| {
                e.value.amount = e.value.amount.saturating_sub(&fee.0.amount);
            });
        }

        println!("change outputs after fee: {:?}", self.change_outputs);
    }

    // /// Calculates the total balance by summing up the balance of all actions and change outputs.
    // fn calculate_balance(&self) -> Balance {
    //     println!("entered calculate_balance!");
    //     let mut balance = Balance::zero();
    //     println!("actions 1: {:?}", self.actions);
    //     for action in &self.actions {
    //         balance += action.balance();
    //     }
    //     println!("baa;lance after action 1: {:?}", balance);
    //     println!("actions 2: {:?}", self.change_outputs.values());
    //     for action in self.change_outputs.values() {
    //         balance += action.balance();
    //     }

    //     println!("balance after action 2: {:?}", balance);

    //     balance
    // }

    // fn calculate_balance_with_fees(&self, base_fee_estimation: Fee) -> Balance {
    //     println!("entered calculate_balance_with_fee!");
    //     let mut balance = Balance::zero();
    //     println!("actions 1: {:?}", self.actions);

    //     // we'll add another spend note here.
    //     for action in &self.actions {
    //         balance += action.balance();
    //     }

    //     println!("baa;lance after action 1: {:?}", balance);
    //     println!("actions 2: {:?}", self.change_outputs.values());
    //     for action in self.change_outputs.values() {
    //         balance += action.balance();
    //     }

    //     println!("balance after action 2: {:?}", balance);

    //     println!("base_fee_estimation.0: {:?}", base_fee_estimation.0);

    //     balance -= base_fee_estimation.0;
    //     println!("balance after fee subtraction: {:?}", balance);

    //     balance
    // }

    // fn push(&mut self, action: ActionPlan) {
    //     self.actions.push(action);
    // }

    // fn gas_estimate(&self) -> Gas {
    //     // TODO: this won't include the gas cost for the bytes of the tx itself
    //     // so this gas estimate will be an underestimate, but since the tx-bytes contribution
    //     // to the fee is ideally small, hopefully it doesn't matter.
    //     let mut gas = Gas::zero();
    //     for action in &self.actions {
    //         // TODO missing AddAssign
    //         gas = gas + action.gas_cost();
    //     }
    //     for action in self.change_outputs.values() {
    //         // TODO missing AddAssign
    //         // TODO missing GasCost impl on OutputPlan
    //         gas = gas + ActionPlan::from(action.clone()).gas_cost();
    //     }

    //     println!("gas is: {:?}", gas);
    //     println!("self.actions is: {:?}", self.actions);
    //     println!("self.change_outputs is: {:?}", self.change_outputs);
    //     println!(")))))))))))))))))");

    //     gas
    // }

    // fn fee_estimate(&self, gas_prices: &GasPrices, fee_tier: &FeeTier) -> Fee {
    //     println!("!!!!!!!!!!!!!!!!! fee_estimate!");
    //     println!("gas_prices in fee_estomate: {:?}", gas_prices);
    //     let base_fee: Fee = Fee::from_staking_token_amount(gas_prices.fee(&self.gas_estimate()));
    //     println!("base fee: {:?}", base_fee);
    //     base_fee.apply_tier(*fee_tier)
    // }

    // fn refresh_change(&mut self, change_address: Address) {
    //     println!("entered refresh_chnage!");
    //     self.change_outputs = BTreeMap::new();
    //     // For each "provided" balance component, create a change note.
    //     for value in self.calculate_balance().provided() {
    //         println!("value is: {:?}", value);
    //         self.change_outputs.insert(
    //             value.asset_id,
    //             OutputPlan::new(&mut OsRng, value, change_address),
    //         );
    //     }

    //     println!("self.change_outputs is: {:?}", self.change_outputs);
    // }

    // fn adjust_change_for_fee(&mut self, fee: Fee) {
    //     println!("self.change_outputs.is_empty(): {:?}", self.change_outputs.is_empty());
    //     if !(self.change_outputs.is_empty()) {
    //         self.change_outputs.entry(fee.0.asset_id).and_modify(|e| {
    //             e.value.amount = e.value.amount.saturating_sub(&fee.0.amount);
    //         });
    //     }
    // }

    /// Prioritize notes to spend to release value of a specific transaction.
    ///
    /// Various logic is possible for note selection. Currently, this method
    /// prioritizes notes sent to a one-time address, then notes with the largest
    /// value:
    ///
    /// - Prioritizing notes sent to one-time addresses optimizes for a future in
    /// which we implement DAGSync keyed by fuzzy message detection (which will not
    /// be able to detect notes sent to one-time addresses). Spending these notes
    /// immediately converts them into change notes, sent to the default address for
    /// the users' account, which are detectable.
    ///
    /// - Prioritizing notes with the largest value optimizes for gas used by the
    /// transaction.
    ///
    /// We may want to make note prioritization configurable in the future. For
    /// instance, a user might prefer a note prioritization strategy that harvested
    /// capital losses when possible, using cost basis information retained by the
    /// view server.
    fn prioritize_and_filter_spendable_notes(
        records: Vec<SpendableNoteRecord>,
    ) -> Vec<SpendableNoteRecord> {
        // Filter out zero valued notes.
        let mut filtered = records
            .into_iter()
            .filter(|record| record.note.amount() > Amount::zero())
            .collect::<Vec<_>>();

        filtered.sort_by(|a, b| b.note.amount().cmp(&a.note.amount()));

        filtered
    }

    /// Set the current gas prices for fee prediction.
    #[instrument(skip(self))]
    pub fn set_gas_prices(&mut self, gas_prices: GasPrices) -> &mut Self {
        self.gas_prices = gas_prices;
        self
    }

    /// Set the fee tier.
    #[instrument(skip(self))]
    pub fn set_fee_tier(&mut self, fee_tier: FeeTier) -> &mut Self {
        self.fee_tier = fee_tier;
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

    /// Spend a specific positioned note in the transaction.
    ///
    /// If you don't use this method to specify spends, they will be filled in automatically from
    /// the view service when the plan is [`finish`](Planner::finish)ed.
    #[instrument(skip(self))]
    pub fn spend(&mut self, note: Note, position: tct::Position) -> &mut Self {
        println!("entered spend!");

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
    ///
    /// Note: Currently this only supports an initial withdrawal from Closed, with no rewards.
    #[instrument(skip(self))]
    pub fn position_withdraw(
        &mut self,
        position_id: position::Id,
        reserves: Reserves,
        pair: TradingPair,
    ) -> &mut Self {
        self.action(ActionPlan::PositionWithdraw(PositionWithdrawPlan {
            reserves,
            position_id,
            pair,
            sequence: 0,
            rewards: Vec::new(),
        }));
        self
    }

    /// Perform a swap claim based on an input swap NFT with a pre-paid fee.
    #[instrument(skip(self))]
    pub fn swap_claim(&mut self, plan: SwapClaimPlan) -> &mut Self {
        println!("entered swap_claim!");

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
        println!("entered swap!");

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
    pub fn delegate(
        &mut self,
        epoch: Epoch,
        unbonded_amount: Amount,
        rate_data: RateData,
    ) -> &mut Self {
        let delegation = rate_data.build_delegate(epoch, unbonded_amount).into();
        self.action(delegation);
        self
    }

    /// Add an undelegation to this transaction.
    ///
    /// TODO: can we put the chain parameters into the planner at the start, so we can compute end_epoch_index?
    #[instrument(skip(self))]
    pub fn undelegate(
        &mut self,
        epoch: Epoch,
        delegation_amount: Amount,
        rate_data: RateData,
    ) -> &mut Self {
        let undelegation = rate_data.build_undelegate(epoch, delegation_amount).into();
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
        self.action(ActionPlan::Ics20Withdrawal(withdrawal));
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

        self.push(vote);
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
        println!(
            "self.plan.actions.clone() original: {:?}",
            self.plan.actions.clone()
        );
        println!("self.balance: {:?}", self.balance);

        // Gather all the information needed from the view service.
        let app_params = view.app_params().await?;
        let chain_id = app_params.chain_id.clone();
        let fmd_params = view.fmd_parameters().await?;

        // Caller has already processed all the user-supplied intents into complete action plans.
        self.actions = self.plan.actions.clone();

        // Change address represents the sender's address.
        let change_address = view.address_by_index(source).await?.clone();

        // Query voting notes.
        let mut voting_notes = Vec::new();
        let mut spendable_notes = Vec::new();
        let (spendable_requests, voting_requests) = self.notes_requests(source);
        for request in voting_requests {
            let notes = view.notes_for_voting(request).await?;
            voting_notes.push(notes);
        }
        // TODO: remove spendable notes
        for request in spendable_requests {
            let notes = view.notes(request).await?;
            spendable_notes.extend(notes);
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
        ) in voting_notes
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
                let unbonded_amount = rate_data.unbonded_amount(record.note.amount()).into();

                // If the delegation token is unspent, "roll it over" by spending it (this will
                // result in change sent back to us). This unlinks nullifiers used for voting on
                // multiple non-overlapping proposals, increasing privacy.
                if record.height_spent.is_none() {
                    self.push(
                        SpendPlan::new(&mut OsRng, record.note.clone(), record.position).into(),
                    );
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

        // // Check enum for voting-based action
        // let mut is_voting = false;
        // for action in self.actions.iter() {
        //     if matches!(action, ActionPlan::Spend(_)) {
        //         is_voting = true;
        //     }
        // }

        // Check enum for voting-based action
        let mut is_swap_claim = false;
        for action in self.actions.iter() {
            if matches!(action, ActionPlan::SwapClaim(_)) {
                is_swap_claim = true;
            }
        }

        println!("self.calculate_balance(): {:?}", self.calculate_balance());

        // new data structure that needs to be explained.
        let mut notes_by_asset_id: Vec<BTreeMap<asset::Id, Vec<SpendableNoteRecord>>> = Vec::new();

        // Cache the balance calculations to avoid multiple calls
        // let balance = self.calculate_balance();
        // let mut required_iter = balance.required().peekable();
        // let mut provided_iter = balance.provided().peekable();

        // // Determine which iterator to use based on the presence of elements
        // let balance_iter: Box<dyn Iterator<Item = penumbra_asset::Value> + Send> =
        // if required_iter.peek().is_some() {
        //     println!("+++++++++++++++++++++++++++++++++++++++++++");
        //     Box::new(required_iter)
        // } else if provided_iter.peek().is_some() {
        //     println!("???????????????????????????????");
        //     Box::new(provided_iter)
        // } else {
        //     // Handle the case where neither iterator has elements
        //     println!("------------------------------------");
        //     Box::new(std::iter::empty::<penumbra_asset::Value>()) as Box<dyn Iterator<Item = _> + Send>
        // };

        for required in self.calculate_balance().required().next() {
            println!("iter 1 is: {:?}", required);
            // create new BTreeMap
            let mut new_map = BTreeMap::new();

            // Find all the notes of this asset in the source account.
            let records: Vec<SpendableNoteRecord> = view
                .notes(NotesRequest {
                    include_spent: false,
                    asset_id: Some(required.asset_id.into()),
                    address_index: Some(source.into()),
                    amount_to_spend: None,
                })
                .await?;

            println!("records is: {:?}", records);

            for record in &records {
                println!(
                    "record.note.value().amount: {:?}",
                    record.note.value().amount
                );
                // if record.note.value().amount == 0 {
                //     println!("zero note detected ======================================================================================================");
                // }
            }

            new_map.insert(
                required.asset_id,
                Self::prioritize_and_filter_spendable_notes(records),
            );

            // Now append this map to the vector
            notes_by_asset_id.push(new_map);
        }

        // if !is_swap_claim {
            // we need to NOW check if we added any of the staking token notes in order to have funds to pay for fees
            // 100% need this or everything will fail
            for notes in notes_by_asset_id.clone() {
                if !notes.contains_key(&*STAKING_TOKEN_ASSET_ID) {
                    println!("does not contain STAKING_TOKEN_ASSET_ID!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                    // create new BTreeMap
                    let mut new_map = BTreeMap::new();

                    // Find all the notes of this asset in the source account.
                    let records: Vec<SpendableNoteRecord> = view
                        .notes(NotesRequest {
                            include_spent: false,
                            asset_id: Some(STAKING_TOKEN_DENOM.id().into()),
                            address_index: Some(source.into()),
                            amount_to_spend: None,
                        })
                        .await?;

                    println!("records is: {:?}", records);

                    for record in &records {
                        println!(
                            "record.note.value().amount: {:?}",
                            record.note.value().amount
                        );
                        // if record.note.value().amount == 0 {
                        //     println!("zero note detected ======================================================================================================");
                        // }
                    }

                    new_map.insert(
                        STAKING_TOKEN_DENOM.id().into(),
                        Self::prioritize_and_filter_spendable_notes(records),
                    );

                    // Now append this map to the vector
                    notes_by_asset_id.push(new_map);
                }
            }

            if notes_by_asset_id.is_empty() {
                println!(
                    "does not contain STAKING_TOKEN_ASSET_ID!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"
                );
                // create new BTreeMap
                let mut new_map = BTreeMap::new();

                // Find all the notes of this asset in the source account.
                let records: Vec<SpendableNoteRecord> = view
                    .notes(NotesRequest {
                        include_spent: false,
                        asset_id: Some(STAKING_TOKEN_DENOM.id().into()),
                        address_index: Some(source.into()),
                        amount_to_spend: None,
                    })
                    .await?;

                println!("records is: {:?}", records);

                for record in &records {
                    println!(
                        "record.note.value().amount: {:?}",
                        record.note.value().amount
                    );
                    // if record.note.value().amount == 0 {
                    //     println!("zero note detected ======================================================================================================");
                    // }
                }

                new_map.insert(
                    STAKING_TOKEN_DENOM.id().into(),
                    Self::prioritize_and_filter_spendable_notes(records),
                );

                // Now append this map to the vector
                notes_by_asset_id.push(new_map);
            }
        // }

        println!("notes_by_asset_id: {:?}", notes_by_asset_id);

        // Calculate initial transaction fees.
        // let mut fee = self.fee_estimate(&self.gas_prices, &self.fee_tier);
        // Set non-zero gas price.
        let mut gas_price = GasPrices::default();
        gas_price.block_space_price = 5u64;
        gas_price.compact_block_space_price = 5u64;
        gas_price.execution_price = 5u64;
        gas_price.verification_price = 5u64;
        let fee_tier = FeeTier::High;

        self.set_gas_prices(gas_price).set_fee_tier(fee_tier);

        let mut fee: Fee = self.fee_estimate(&self.gas_prices, &self.fee_tier);

        println!("fee: {:?}", fee);

        // size of the vector
        let notes_by_asset_id_size = notes_by_asset_id.len();
        println!("notes_by_asset_id_size: {:?}", notes_by_asset_id_size);

        // Cache the balance calculations to avoid multiple calls
        let balance = self.calculate_balance_with_fees(fee);
        println!(
            "check self.calculate_balance_with_fees(fee): {:?}",
            self.calculate_balance_with_fees(fee)
        );

        ////////////////////////////////////
        /// provided
        /// TODO: is this neccessary?
        while let Some(required) = self.calculate_balance().provided().next() {
            // Recompute the change outputs, without accounting for fees.
            self.refresh_change(change_address.clone());

            // Now re-estimate the fee of the updated transaction and adjust the change if possible.
            fee = self.fee_estimate(&self.gas_prices, &self.fee_tier);
            println!("fee estimate: {:?}", fee);

            // self.adjust_change_for_fee(fee);

            // Need to account to balance after applying fees.
            self.balance = self.calculate_balance_with_fees(fee);
            // self.balance = self.calculate_balance();

            // let dimension: usize = self.calculate_balance().dimension();
            // println!("dimension is: {:?}", dimension);
            // println!(
            //     "otes_by_asset_id_size - 1 is: {:?}",
            //     notes_by_asset_id_size - 1
            // );

            // // this means we've handled one iteration successfully
            // // don't consume the iterator
            // if notes_by_asset_id_size - 1 == dimension {
            //     println!("need to iterate!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
            //     index += 1;
            // }
        }
        //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

        // TODO: fix this damn iterator!
        // let mut required_iter = balance.required().peekable();
        // let mut provided_iter = balance.provided().peekable();

        // // Determine which iterator to use based on the presence of elements
        // let mut balance_iter: Box<dyn Iterator<Item = penumbra_asset::Value> + Send> =
        // if required_iter.peek().is_some() {
        //     println!("***********************************************");
        //     Box::new(required_iter)
        // } else if provided_iter.peek().is_some() {
        //     println!("^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^");
        //     Box::new(provided_iter)
        // } else {
        //     println!("&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&");
        //     // Handle the case where neither iterator has elements with empty iterator
        //     Box::new(std::iter::empty::<penumbra_asset::Value>())
        //         as Box<dyn Iterator<Item = _> + Send>
        // };

        // Add spends and change outputs as required to balance the transaction, using the spendable
        // notes provided. It is the caller's responsibility to ensure that the notes are the result of
        // collected responses to the requests generated by an immediately preceding call to
        // [`Planner::note_requests`].
        let mut iterations = 0usize;
        let mut index: usize = 0;
        while let Some(required) = self.calculate_balance_with_fees(fee).required().next() {
            println!("self.actions 1: {:?}", self.actions);
            println!("required is: {:?}", required);
            println!(
                "1 self.calculate_balance_with_fees(fee): {:?}",
                self.calculate_balance_with_fees(fee)
            );
            // Spend a single note towards the required balance, if possible.
            // This adds the required spends to the planner.
            println!("required.asset_id: {:?}", required.asset_id);

            // If it's a swap claim, handle it differently
            //  if is_swap_claim {
            //     let records: Vec<SpendableNoteRecord> = view
            //     .notes(NotesRequest {
            //         include_spent: false,
            //         asset_id: Some(required.asset_id.into()),
            //         address_index: Some(source.into()),
            //         amount_to_spend: None,
            //     })
            //     .await?;

            //     println!("records is: {:?}", records);

            //     notes_by_asset_id.insert(
            //         required.asset_id,
            //         Self::prioritize_and_filter_spendable_notes(records),
            //     );
            // }

            // this will fail for swap_claims!
            // let mut zero_amount_records = Vec::new();
            // if !is_swap_claim {
            // let Some((asset_id, mut note)) = notes_by_asset_id[index].pop_first()
            // // let Some(note) = notes_by_asset_id
            //     // .get_mut(&required.asset_id)
            //     // .expect("we already queried")
            //     // .pop()
            // else {
            //     return Err(anyhow!(
            //         "ran out of notes to spend while planning transaction, need {} of asset {}",
            //         required.amount,
            //         required.asset_id,
            //     )
            //     .into());
            // };

            // Spend a single note towards the required balance, if possible.
            // This adds the required spends to the planner.
            // TODO: get_mut may not get the largest note that we're already spent time filtering for.
            let Some(note) = notes_by_asset_id[index]
                .get_mut(&required.asset_id)
                .expect("we already queried")
                .pop()
            else {
                return Err(anyhow!(
                    "ran out of notes to spend while planning transaction, need {} of asset {}",
                    required.amount,
                    required.asset_id,
                )
                .into());
            };

            // zero_amount_records.push(note.clone());
            // zero_amount_records.push(note[0].clone());
            // }

            // push a staking token note
            // let Some((asset_id_fee, mut note_fee)) = staking_token_notes_for_fees.pop_first()
            //     // .get_mut(&required.asset_id)
            //     // .expect("we already queried")
            //     // .pop()
            // else {
            //     return Err(anyhow!(
            //         "ran out of notes to spend while planning transaction, need {} of asset {}",
            //         required.amount,
            //         required.asset_id,
            //     )
            //     .into());
            // };

            // Add the required spends to the planner.
            // if !is_swap_claim {
            self.push(SpendPlan::new(&mut OsRng, note.clone().note, note.clone().position).into());
            // }

            println!("self.actions 1.5: {:?}", self.actions);

            // self.push(SpendPlan::new(&mut OsRng, note_fee[0].clone().note, note_fee[0].clone().position).into());

            // Recompute the change outputs, without accounting for fees.
            self.refresh_change(change_address.clone());

            // Now re-estimate the fee of the updated transaction and adjust the change if possible.
            fee = self.fee_estimate(&self.gas_prices, &self.fee_tier);
            println!("fee estimate: {:?}", fee);

            self.adjust_change_for_fee(fee);

            // Need to account to balance after applying fees.
            self.balance = self.calculate_balance_with_fees(fee);
            // self.balance = self.calculate_balance();

            println!("self.actions 2: {:?}", self.actions);

            println!(
                "2 self.calculate_balance_with_fees(fee): {:?}",
                self.calculate_balance_with_fees(fee)
            );

            let dimension: usize = self.calculate_balance_with_fees(fee).dimension();
            println!("dimension is: {:?}", dimension);
            println!(
                "otes_by_asset_id_size - 1 is: {:?}",
                notes_by_asset_id_size - 1
            );

            // this means we've handled one iteration successfully
            // don't consume the iterator
            if notes_by_asset_id_size - 1 == dimension {
                println!("need to iterate!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                index += 1;
            }

            // println!("elf.balance.provided().next() is: {:?}", self.balance.provided().next().unwrap().amount);

            // We've successfully balanced the equation.
            // if self.balance.provided().next().unwrap().amount == 0u64.into() {
            //     break;
            // }
            println!("required end is: {:?}", required);

            if self.balance.is_zero() {
                println!("self.balance is zero!");
                break;
            }

            iterations += 1;
            if iterations > 100 {
                return Err(anyhow!("failed to plan transaction after 100 iterations").into());
            }
        }

        // TODO: verify the provided case

        // TODO: For any remaining provided balance, make a single change note for each
        // for value in self.balance.provided().collect::<Vec<_>>() {
        //     self.push(OutputPlan::new(&mut OsRng, value, change_address).into());
        // }

        println!("continue hell!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");

        println!("we've balanced the fees!");

        // TODO: check if swap claims need to enter the loop and pay fees?

        ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // everything here is great
        ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

        let fee = self.fee_estimate(&self.gas_prices, &self.fee_tier);

        // Assemble the fully-formed transaction plan.
        self.plan = TransactionPlan {
            actions: self
                .actions
                .clone()
                .into_iter()
                .chain(self.change_outputs.clone().into_values().map(Into::into))
                .collect(),
            transaction_parameters: TransactionParameters {
                expiry_height: self.plan.transaction_parameters.expiry_height,
                chain_id: chain_id.clone(),
                fee: fee,
            },
            detection_data: None,
            memo: self.plan.memo.clone(),
        };

        // If there are outputs, we check that a memo has been added. If not, we add a blank memo.
        if self.plan.num_outputs() > 0 && self.plan.memo.is_none() {
            self.memo(MemoPlaintext::blank_memo(change_address.clone()))
                .expect("empty string is a valid memo");
        } else if self.plan.num_outputs() == 0 && self.plan.memo.is_some() {
            anyhow::bail!("if no outputs, no memo should be added");
        }

        // Add clue plans for `Output`s.
        self.plan
            .populate_detection_data(&mut OsRng, fmd_params.precision_bits.into());

        // All actions have now been added, so check to make sure that you don't build and submit an
        // empty transaction.
        if self.actions.is_empty() {
            anyhow::bail!("planned transaction would be empty, so should not be submitted");
        }

        // Now the transaction should be fully balanced, unless we didn't have enough to spend
        if !self.calculate_balance_with_fees(fee.clone()).is_zero() {
            anyhow::bail!(
                "balance is non-zero after attempting to balance transaction: {:?}",
                self.balance
            );
        }

        tracing::debug!(plan = ?self.plan, "finished balancing transaction");

        // Clear the contents of the planner, which can be re-used.
        self.balance = Balance::zero();
        self.vote_intents = BTreeMap::new();
        self.gas_prices = GasPrices::zero();
        self.actions = Vec::new();
        self.change_outputs = BTreeMap::new();

        // clean note by asset id
        notes_by_asset_id = Vec::new();
        let plan = mem::take(&mut self.plan);

        Ok(plan)
    }
}
