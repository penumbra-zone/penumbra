use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Formatter},
    mem,
};

use anyhow::{anyhow, Result};
use indexed_db_futures::{IdbDatabase, IdbQuerySource};
use indexed_db_futures::prelude::OpenDbRequest;
use rand_core::{CryptoRng, OsRng, RngCore};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use penumbra_asset::{asset::DenomMetadata, Balance, Value};
use penumbra_chain::params::{ChainParameters, FmdParameters};
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
use penumbra_fee::Fee;
use penumbra_keys::{
    keys::{AccountGroupId, AddressIndex},
    Address,
};
use penumbra_num::Amount;
use penumbra_proto::view::v1alpha1::{NotesForVotingRequest, NotesRequest};
use penumbra_shielded_pool::{Note, OutputPlan, SpendPlan};
use penumbra_stake::{rate::RateData, validator};
use penumbra_stake::{IdentityKey, UndelegateClaimPlan};
use penumbra_tct as tct;
use penumbra_transaction::{
    action::{
        Proposal, ProposalDepositClaim, ProposalSubmit, ProposalWithdraw, ValidatorVote, Vote,
    },
    memo::MemoPlaintext,
    plan::{ActionPlan, DelegatorVotePlan, MemoPlan, TransactionPlan},
    proposal,
};
use tracing::instrument;
use crate::note_record::SpendableNoteRecord;


/// A planner for a [`TransactionPlan`] that can fill in the required spends and change outputs upon
/// finalization to make a transaction balance.
#[wasm_bindgen]
pub struct WasmPlanner {
    rng: OsRng,
    balance: Balance,
    vote_intents: BTreeMap<u64, VoteIntent>,
    plan: TransactionPlan,
    // IMPORTANT: if you add more fields here, make sure to clear them when the planner is finished
}

#[derive(Debug, Clone)]
struct VoteIntent {
    start_block_height: u64,
    start_position: tct::Position,
    rate_data: BTreeMap<IdentityKey, RateData>,
    vote: Vote,
}

impl Debug for WasmPlanner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("balance", &self.balance)
            .field("plan", &self.plan)
            .finish()
    }
}

#[wasm_bindgen]
impl WasmPlanner {
    /// Create a new planner.
    ///
    pub fn new() -> WasmPlanner {
        Self {
            rng: OsRng::default(),
            balance: Balance::default(),
            vote_intents: BTreeMap::default(),
            plan: TransactionPlan::default(),
        }
    }


    /// Set the expiry height for the transaction plan.
    pub fn expiry_height(&mut self, expiry_height: u64) {
        self.plan.expiry_height = expiry_height;
    }

    /// Set a memo for this transaction plan.
    ///
    /// Errors if the memo is too long.
    pub fn memo(&mut self, memo: JsValue) {
        let memo_plain_text: penumbra_proto::core::transaction::v1alpha1::MemoPlaintext = serde_wasm_bindgen::from_value(memo).unwrap();
        self.plan.memo_plan = Some(MemoPlan::new(&mut self.rng, memo_plain_text.try_into().unwrap()).unwrap());
    }

    /// Add a fee to the transaction plan.
    ///
    /// This function should be called once.
    pub fn fee(&mut self, fee: JsValue) {
        self.balance += serde_wasm_bindgen::from_value(fee.clone()).0;
        self.plan.fee = serde_wasm_bindgen::from_value(fee).unwrap();
    }

    /// Spend a specific positioned note in the transaction.
    ///
    /// If you don't use this method to specify spends, they will be filled in automatically from
    /// the view service when the plan is [`finish`](WasmPlanner::finish)ed.
    pub fn spend(&mut self, note: JsValue, position: JsValue) {
        let note_proto: penumbra_proto::core::crypto::v1alpha1::Note = serde_wasm_bindgen::from_value(note).unwrap();
        let position_proto: penumbra_proto::core::crypto::v1alpha1:: = serde_wasm_bindgen::from_value(note).unwrap();

        let spend = SpendPlan::new(&mut self.rng, note_proto.try_into(), position).into();
        self.action(spend);
    }

    /// Open a liquidity position in the order book.
    // pub fn position_open(&mut self, position: Position) {
    //     self.action(ActionPlan::PositionOpen(PositionOpen { position }));
    // }

    /// Close a liquidity position in the order book.
    // pub fn position_close(&mut self, position_id: position::Id) {
    //     self.action(ActionPlan::PositionClose(PositionClose { position_id }));
    // }

    /// Withdraw a liquidity position in the order book.
    // pub fn position_withdraw(
    //     &mut self,
    //     position_id: position::Id,
    //     reserves: Reserves,
    //     pair: TradingPair,
    // ) {
    //     self.action(ActionPlan::PositionWithdraw(PositionWithdrawPlan::new(
    //         reserves,
    //         position_id,
    //         pair,
    //     )));
    // }

    /// Perform a swap claim based on an input swap NFT with a pre-paid fee.
    // pub fn swap_claim(&mut self, plan: SwapClaimPlan) {
    //     // Nothing needs to be spent, since the fee is pre-paid and the
    //     // swap NFT will be automatically consumed when the SwapClaim action
    //     // is processed by the validators.
    //     // TODO: need to set the intended fee so the tx actually balances,
    //     // otherwise the planner will create an output
    //     self.action(plan.into());
    // }

    /// Perform a swap based on input notes in the transaction.
    // pub fn swap(
    //     &mut self,
    //     input_value: Value,
    //     into_denom: DenomMetadata,
    //     swap_claim_fee: Fee,
    //     claim_address: Address,
    // ) {
    //     // Determine the canonical order for the assets being swapped.
    //     // This will determine whether the input amount is assigned to delta_1 or delta_2.
    //     let trading_pair = TradingPair::new(input_value.asset_id, into_denom.id());
    //
    //     // If `trading_pair.asset_1` is the input asset, then `delta_1` is the input amount,
    //     // and `delta_2` is 0.
    //     //
    //     // Otherwise, `delta_1` is 0, and `delta_2` is the input amount.
    //     let (delta_1, delta_2) = if trading_pair.asset_1() == input_value.asset_id {
    //         (input_value.amount, 0u64.into())
    //     } else {
    //         (0u64.into(), input_value.amount)
    //     };
    //
    //     // If there is no input, then there is no swap.
    //     if delta_1 == Amount::zero() && delta_2 == Amount::zero() {
    //     }
    //
    //     // Create the `SwapPlaintext` representing the swap to be performed:
    //     let swap_plaintext = SwapPlaintext::new(
    //         &mut self.rng,
    //         trading_pair,
    //         delta_1,
    //         delta_2,
    //         swap_claim_fee,
    //         claim_address,
    //     );
    //
    //     let swap = SwapPlan::new(&mut self.rng, swap_plaintext).into();
    //     self.action(swap);
    // }

    /// Add an output note from this transaction.
    ///
    /// Any unused output value will be redirected back to the originating address as change notes
    /// when the plan is [`finish`](Builder::finish)ed.
    pub fn output(&mut self, value: JsValue, address: JsValue) {
        let output = OutputPlan::new(&mut self.rng, serde_wasm_bindgen::from_value(value).unwrap(),
                                     serde_wasm_bindgen::from_value(address).unwrap()).into();
        self.action(output);
    }

    /// Add a delegation to this transaction.
    ///
    /// If you don't specify spends or outputs as well, they will be filled in automatically.
    // pub fn delegate(&mut self, unbonded_amount: u128, rate_data: RateData) {
    //     let delegation = rate_data.build_delegate(unbonded_amount).into();
    //     self.action(delegation);
    // }

    /// Add an undelegation to this transaction.
    ///
    /// TODO: can we put the chain parameters into the planner at the start, so we can compute end_epoch_index?
    // pub fn undelegate(&mut self, delegation_amount: Amount, rate_data: RateData) {
    //     let undelegation = rate_data.build_undelegate(delegation_amount).into();
    //     self.action(undelegation);
    // }

    /// Add an undelegate claim to this transaction.
    // pub fn undelegate_claim(&mut self, claim_plan: UndelegateClaimPlan) {
    //     self.action(ActionPlan::UndelegateClaim(claim_plan));
    // }

    /// Upload a validator definition in this transaction.
    // pub fn validator_definition(&mut self, new_validator: validator::Definition) {
    //     self.action(ActionPlan::ValidatorDefinition(new_validator.into()));
    // }

    /// Submit a new governance proposal in this transaction.
    // pub fn proposal_submit(&mut self, proposal: Proposal, deposit_amount: Amount) {
    //     self.action(ActionPlan::ProposalSubmit(ProposalSubmit {
    //         proposal,
    //         deposit_amount,
    //     }));
    // }

    /// Withdraw a governance proposal in this transaction.
    // pub fn proposal_withdraw(&mut self, proposal: u64, reason: String) {
    //     self.action(ActionPlan::ProposalWithdraw(ProposalWithdraw {
    //         proposal,
    //         reason,
    //     }));
    // }

    /// Claim a governance proposal deposit in this transaction.
    // pub fn proposal_deposit_claim(
    //     &mut self,
    //     proposal: u64,
    //     deposit_amount: Amount,
    //     outcome: proposal::Outcome<()>,
    // ) {
    //     self.action(ActionPlan::ProposalDepositClaim(ProposalDepositClaim {
    //         proposal,
    //         deposit_amount,
    //         outcome,
    //     }));
    // }

    /// Cast a validator vote in this transaction.
    // pub fn validator_vote(&mut self, vote: ValidatorVote) {
    //     self.action(ActionPlan::ValidatorVote(vote));
    // }

    /// Vote with all possible vote weight on a given proposal.
    ///
    /// Voting twice on the same proposal in the same planner will overwrite the previous vote.
    // pub fn delegator_vote(
    //     &mut self,
    //     proposal: u64,
    //     start_block_height: u64,
    //     start_position: tct::Position,
    //     start_rate_data: BTreeMap<IdentityKey, RateData>,
    //     vote: Vote,
    // ) {
    //     self.vote_intents.insert(
    //         proposal,
    //         VoteIntent {
    //             start_position,
    //             start_block_height,
    //             vote,
    //             rate_data: start_rate_data,
    //         },
    //     );
    // }

    /// Vote with a specific positioned note in the transaction.
    ///
    /// If you don't use this method to specify votes, they will be filled in automatically from the
    /// implied voting intent by [`vote`](WasmPlanner::vote) when the plan is
    /// [`finish`](WasmPlanner::finish)ed.
    // pub fn delegator_vote_precise(
    //     &mut self,
    //     proposal: u64,
    //     start_position: tct::Position,
    //     vote: Vote,
    //     note: Note,
    //     position: tct::Position,
    //     unbonded_amount: Amount,
    // ) {
    //     let vote = DelegatorVotePlan::new(
    //         &mut self.rng,
    //         proposal,
    //         start_position,
    //         vote,
    //         note,
    //         position,
    //         unbonded_amount,
    //     )
    //         .into();
    //     self.action(vote);
    // }


    pub async fn plan(
        &mut self,
        self_address: JsValue,
    ) -> anyhow::Result<TransactionPlan> {
        let address: Address = serde_wasm_bindgen::from_value(self_address).unwrap();
        // Gather all the information needed from the view service
        let chain_params = get_chain_parameters().await.expect("Error getting ChainParameters");
        let fmd_params = get_fmd_parameters().await.expect("Error getting FmdParameters");
        let mut spendable_notes = Vec::new();
        let mut voting_notes = Vec::new();


        self.plan_with_spendable_and_votable_notes(
            &chain_params,
            &fmd_params,
            spendable_notes,
            voting_notes,
            address,
        )
    }
}

impl WasmPlanner {
    pub fn balance(&self) -> &Balance {
        &self.balance
    }

    fn action(&mut self, action: ActionPlan) {
        // Track the contribution of the action to the transaction's balance
        self.balance += action.balance();

        // Add the action to the plan
        self.plan.actions.push(action);
    }


    /// Add spends and change outputs as required to balance the transaction, using the spendable
    /// notes provided. It is the caller's responsibility to ensure that the notes are the result of
    /// collected responses to the requests generated by an immediately preceding call to
    /// [`WasmPlanner::note_requests`].
    ///
    /// Clears the contents of the planner, which can be re-used.

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
        self.plan.chain_id = chain_params.chain_id.clone();

        // Add the required spends to the planner
        for record in spendable_notes {
            self.spend(record.note, record.position);
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
                let Some(rate_data) = rate_data.get(&identity_key) else { continue; };
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
                return Err(anyhow!(
                    "can't vote on proposal {} because no delegation notes were staked to an active validator when voting started",
                    proposal
                ));
            }
        }

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

        // If there are outputs, we check that a memo has been added. If not, we add a default memo.
        if self.plan.num_outputs() > 0 && self.plan.memo_plan.is_none() {
            self.plan.memo_plan = Some(MemoPlan::new(&mut self.rng, MemoPlaintext::default())?)
        } else if self.plan.num_outputs() == 0 && self.plan.memo_plan.is_some() {
            anyhow::bail!("if no outputs, no memo should be added");
        }

        // Add clue plans for `Output`s.
        let precision_bits = fmd_params.precision_bits;
        self.plan
            .add_all_clue_plans(&mut self.rng, precision_bits.into());

        tracing::debug!(plan = ?self.plan, "finished balancing transaction");

        // Clear the planner and pull out the plan to return
        self.balance = Balance::zero();
        self.vote_intents = BTreeMap::new();
        let plan = mem::take(&mut self.plan);

        Ok(plan)
    }
}


pub async fn get_chain_parameters() -> Option<ChainParameters> {
    let db_req: OpenDbRequest = IdbDatabase::open_u32("penumbra", 11).ok()?;

    let db: IdbDatabase = db_req.into_future().await.ok()?;

    let tx = db.transaction_on_one("chain_parameters").ok()?;
    let store = tx.object_store("chain_parameters").ok()?;

    let value = store
        .get_all()
        .ok()?
        .await
        .ok()?.get(0);

    serde_wasm_bindgen::from_value(value).ok()?
}

pub async fn get_fmd_parameters() -> Option<FmdParameters> {
    let db_req: OpenDbRequest = IdbDatabase::open_u32("penumbra", 11).ok()?;

    let db: IdbDatabase = db_req.into_future().await.ok()?;

    let tx = db.transaction_on_one("fmd_parameters").ok()?;
    let store = tx.object_store("fmd_parameters").ok()?;

    let value = store
        .get_owned("fmd")
        .ok()?
        .await
        .ok()?;

    serde_wasm_bindgen::from_value(value?).ok()?
}

