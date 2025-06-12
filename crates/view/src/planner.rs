use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Formatter},
    mem,
};

use anyhow::{Context, Result};
use penumbra_sdk_funding::liquidity_tournament::ActionLiquidityTournamentVotePlan;
use penumbra_sdk_sct::epoch::Epoch;
use rand::{CryptoRng, RngCore};
use rand_core::OsRng;
use tracing::instrument;

use crate::{SpendableNoteRecord, ViewClient};
use anyhow::anyhow;
use penumbra_sdk_asset::{
    asset::{self, Denom},
    Value,
};
use penumbra_sdk_auction::auction::dutch::DutchAuctionDescription;
use penumbra_sdk_auction::auction::dutch::{actions::ActionDutchAuctionWithdrawPlan, DutchAuction};
use penumbra_sdk_auction::auction::{
    dutch::actions::{ActionDutchAuctionEnd, ActionDutchAuctionSchedule},
    AuctionId,
};
use penumbra_sdk_community_pool::CommunityPoolDeposit;
use penumbra_sdk_dex::{
    lp::{
        action::PositionClose,
        plan::{PositionOpenPlan, PositionWithdrawPlan},
        position::{self, Position},
        PositionMetadata, Reserves,
    },
    swap::{SwapPlaintext, SwapPlan},
    swap_claim::SwapClaimPlan,
    TradingPair,
};
use penumbra_sdk_fee::{Fee, FeeTier, GasPrices};
use penumbra_sdk_governance::{
    proposal_state, DelegatorVotePlan, Proposal, ProposalDepositClaim, ProposalSubmit,
    ProposalWithdraw, ValidatorVote, Vote,
};
use penumbra_sdk_ibc::IbcRelay;
use penumbra_sdk_keys::{keys::AddressIndex, Address};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::view::v1::{NotesForVotingRequest, NotesRequest};
use penumbra_sdk_shielded_pool::{Ics20Withdrawal, Note, OutputPlan, SpendPlan};
use penumbra_sdk_stake::{rate::RateData, validator, IdentityKey, UndelegateClaimPlan};
use penumbra_sdk_tct as tct;
use penumbra_sdk_transaction::{
    memo::MemoPlaintext,
    plan::{ActionPlan, MemoPlan, TransactionPlan},
    ActionList, TransactionParameters,
};

/// A planner for a [`TransactionPlan`] that can fill in the required spends and change outputs upon
/// finalization to make a transaction balance.
pub struct Planner<R: RngCore + CryptoRng> {
    rng: R,
    action_list: ActionList,
    /// The fee tier to apply to this transaction.
    fee_tier: FeeTier,
    /// The set of prices used for gas estimation.
    gas_prices: Option<GasPrices>,
    /// The transaction parameters to use for the transaction.
    transaction_parameters: TransactionParameters,
    /// A user-specified change address, if any.
    change_address: Option<Address>,
    /// A user-specified memo text, if any.
    memo_text: Option<String>,
    /// A user-specified memo return address, if any.
    memo_return_address: Option<Address>,
}

impl<R: RngCore + CryptoRng> Debug for Planner<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Planner")
            .field("action_list", &self.action_list)
            .field("fee_tier", &self.fee_tier)
            .field("gas_prices", &self.gas_prices)
            .field("transaction_parameters", &self.transaction_parameters)
            .field("change_address", &self.change_address)
            .field("memo_text", &self.memo_text)
            .field("memo_return_address", &self.memo_return_address)
            .finish()
    }
}

impl<R: RngCore + CryptoRng> Planner<R> {
    /// Create a new planner.
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            action_list: Default::default(),
            gas_prices: Default::default(),
            fee_tier: Default::default(),
            transaction_parameters: Default::default(),
            change_address: None,
            memo_text: None,
            memo_return_address: None,
        }
    }

    /// Add an arbitrary action to the planner.
    pub fn action<A: Into<ActionPlan>>(&mut self, action: A) -> &mut Self {
        self.action_list.push(action);
        self
    }

    /// Set the current gas prices for fee prediction.
    #[instrument(skip(self))]
    pub fn set_gas_prices(&mut self, gas_prices: GasPrices) -> &mut Self {
        self.gas_prices = Some(gas_prices);
        self
    }

    /// Set the fee tier.
    #[instrument(skip(self))]
    pub fn set_fee_tier(&mut self, fee_tier: FeeTier) -> &mut Self {
        self.fee_tier = fee_tier;
        self
    }

    /// Set the expiry height for the transaction.
    #[instrument(skip(self))]
    pub fn expiry_height(&mut self, expiry_height: u64) -> &mut Self {
        self.transaction_parameters.expiry_height = expiry_height;
        self
    }

    /// Set a human-readable memo text for the transaction.
    ///
    /// Errors if the memo is too long.
    #[instrument(skip(self))]
    pub fn memo(&mut self, text: String) -> &mut Self {
        self.memo_text = Some(text);
        self
    }

    /// Customize the return address for the memo.
    ///
    /// If unset, this will default to the address for the source account.  This
    /// must be an address controlled by the user, as the expectation is that
    /// the recipient can use the address to transact with the sender.
    #[instrument(skip(self))]
    pub fn memo_return_address(&mut self, address: Address) -> &mut Self {
        self.memo_return_address = Some(address);
        self
    }

    /// Set the change address for the transaction.
    ///
    /// If unset, this will default to the address for the source account.
    ///
    /// This can be a foreign address, allowing "send max" functionality.
    #[instrument(skip(self))]
    pub fn change_address(&mut self, address: Address) -> &mut Self {
        self.change_address = Some(address);
        self
    }

    /// Spend a specific positioned note in the transaction.
    #[instrument(skip(self))]
    pub fn spend(&mut self, note: Note, position: tct::Position) -> &mut Self {
        self.action_list
            .push(SpendPlan::new(&mut self.rng, note, position));
        self
    }

    /// Add an output note from this transaction.
    ///
    /// Any unused output value will be redirected back to the originating address as change notes.
    #[instrument(skip(self))]
    pub fn output(&mut self, value: Value, address: Address) -> &mut Self {
        self.action_list
            .push(OutputPlan::new(&mut self.rng, value, address));
        self
    }

    /// Open a liquidity position in the order book.
    #[instrument(skip(self))]
    pub fn position_open(&mut self, position: Position) -> &mut Self {
        self.action_list.push(PositionOpenPlan {
            position,
            metadata: Some(PositionMetadata::default()),
        });
        self
    }

    /// Open a liquidity position in the order book.
    #[instrument(skip(self))]
    pub fn position_open_with_metadata(
        &mut self,
        position: Position,
        metadata: PositionMetadata,
    ) -> &mut Self {
        self.action_list.push(PositionOpenPlan {
            position,
            metadata: Some(metadata),
        });
        self
    }

    /// Close a liquidity position in the order book.
    #[instrument(skip(self))]
    pub fn position_close(&mut self, position_id: position::Id) -> &mut Self {
        self.action_list.push(PositionClose { position_id });
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
        next_sequence: u64,
    ) -> &mut Self {
        self.action_list.push(PositionWithdrawPlan {
            reserves,
            position_id,
            pair,
            sequence: next_sequence,
            rewards: Vec::new(),
        });
        self
    }

    /// Schedule a Dutch auction.
    #[instrument(skip(self))]
    pub fn dutch_auction_schedule(&mut self, description: DutchAuctionDescription) -> &mut Self {
        self.action_list
            .push(ActionDutchAuctionSchedule { description });
        self
    }

    /// Ends a Dutch auction.
    #[instrument(skip(self))]
    pub fn dutch_auction_end(&mut self, auction_id: AuctionId) -> &mut Self {
        self.action_list.push(ActionDutchAuctionEnd { auction_id });
        self
    }

    /// Withdraws the reserves of the Dutch auction.
    ///
    /// Uses the provided auction state to automatically end the auction
    /// if necessary.
    #[instrument(skip(self))]
    pub fn dutch_auction_withdraw(&mut self, auction: &DutchAuction) -> &mut Self {
        let auction_id = auction.description.id();
        // Check if the auction needs to be ended
        if auction.state.sequence == 0 {
            self.dutch_auction_end(auction_id);
        }

        let reserves_input = Value {
            amount: auction.state.input_reserves,
            asset_id: auction.description.input.asset_id,
        };
        let reserves_output = Value {
            amount: auction.state.output_reserves,
            asset_id: auction.description.output_id,
        };

        let plan = ActionDutchAuctionWithdrawPlan {
            auction_id,
            seq: 2, // 1 (closed) -> 2 (withdrawn)
            reserves_input,
            reserves_output,
        };

        self.action_list.push(plan);
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

        let swap = SwapPlan::new(&mut self.rng, swap_plaintext);
        self.action_list.push(swap);

        Ok(self)
    }

    /// Perform a swap claim based on an input swap with a pre-paid fee.
    #[instrument(skip(self))]
    pub fn swap_claim(&mut self, plan: SwapClaimPlan) -> &mut Self {
        self.action_list.push(plan);
        self
    }

    /// Add a delegation to this transaction.
    #[instrument(skip(self))]
    pub fn delegate(
        &mut self,
        epoch: Epoch,
        unbonded_amount: Amount,
        rate_data: RateData,
    ) -> &mut Self {
        let delegation = rate_data.build_delegate(epoch, unbonded_amount);
        self.action_list.push(delegation);
        self
    }

    /// Add an undelegation to this transaction.
    #[instrument(skip(self))]
    pub fn undelegate(
        &mut self,
        epoch: Epoch,
        delegation_amount: Amount,
        rate_data: RateData,
    ) -> &mut Self {
        let undelegation = rate_data.build_undelegate(epoch, delegation_amount);
        self.action_list.push(undelegation);
        self
    }

    /// Add an undelegate claim to this transaction.
    #[instrument(skip(self))]
    pub fn undelegate_claim(&mut self, claim_plan: UndelegateClaimPlan) -> &mut Self {
        self.action_list.push(claim_plan);
        self
    }

    /// Upload a validator definition in this transaction.
    #[instrument(skip(self))]
    pub fn validator_definition(&mut self, new_validator: validator::Definition) -> &mut Self {
        self.action_list.push(new_validator);
        self
    }

    /// Submit a new governance proposal in this transaction.
    #[instrument(skip(self))]
    pub fn proposal_submit(&mut self, proposal: Proposal, deposit_amount: Amount) -> &mut Self {
        self.action_list.push(ProposalSubmit {
            proposal,
            deposit_amount,
        });
        self
    }

    /// Withdraw a governance proposal in this transaction.
    #[instrument(skip(self))]
    pub fn proposal_withdraw(&mut self, proposal: u64, reason: String) -> &mut Self {
        self.action_list.push(ProposalWithdraw { proposal, reason });
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
        self.action_list.push(ProposalDepositClaim {
            proposal,
            deposit_amount,
            outcome,
        });
        self
    }

    /// Deposit a value into the Community Pool.
    #[instrument(skip(self))]
    pub fn community_pool_deposit(&mut self, value: Value) -> &mut Self {
        self.action_list.push(CommunityPoolDeposit { value });
        self
    }

    /// Cast a validator vote in this transaction.
    #[instrument(skip(self))]
    pub fn validator_vote(&mut self, vote: ValidatorVote) -> &mut Self {
        self.action_list.push(vote);
        self
    }

    /// Perform an ICS-20 withdrawal
    #[instrument(skip(self))]
    pub fn ics20_withdrawal(&mut self, withdrawal: Ics20Withdrawal) -> &mut Self {
        self.action_list.push(withdrawal);
        self
    }

    /// Perform an IBC action
    #[instrument(skip(self))]
    pub fn ibc_action(&mut self, ibc_action: IbcRelay) -> &mut Self {
        self.action_list.push(ibc_action);
        self
    }

    /// Vote with all possible vote weight on a given proposal.
    #[instrument(skip_all)]
    pub async fn delegator_vote<V: ViewClient>(
        // TODO this sucks, why isn't there a bundle of proposal data to use for voting
        // how is that not the thing returned by the rpc? why do we have to query a bunch of shit
        // independently and stitch it together?
        &mut self,
        view: &mut V,
        source: AddressIndex,
        proposal: u64,
        vote: Vote,
        start_block_height: u64,
        start_position: tct::Position,
        start_rate_data: BTreeMap<IdentityKey, RateData>,
    ) -> Result<&mut Self, anyhow::Error> {
        let voting_notes = view
            .notes_for_voting(NotesForVotingRequest {
                votable_at_height: start_block_height,
                address_index: Some(source.into()),
            })
            .await?;

        anyhow::ensure!(
            !voting_notes.is_empty(),
            "no notes were found for voting on proposal {}",
            proposal
        );

        // 1. Create a DelegatorVotePlan for each votable note.
        for (record, ik) in &voting_notes {
            let Some(validator_start_rate_data) = start_rate_data.get(&ik) else {
                tracing::debug!("missing rate data for votable note delegated to {}", ik);
                continue;
            };

            let voting_power_at_vote_start =
                validator_start_rate_data.unbonded_amount(record.note.amount());

            // 1. Create a DelegatorVotePlan that votes with this note on the proposal.
            let plan = DelegatorVotePlan::new(
                &mut self.rng,
                proposal,
                start_position,
                vote,
                record.note.clone(),
                record.position,
                voting_power_at_vote_start,
            );
            self.delegator_vote_precise(plan);
        }

        // 2. Here, we could sweep any spendable notes with delegation tokens to
        // a new output to try to unlink them from a future vote.  In practice
        // this is meaningless because we don't have flow encryption, so
        // delegator votes reveal the precise amount, and this amount will
        // likely be unique to the delegator and enough to link their votes.
        // Also, because we're in a single transaction, the pattern of
        // delegations will also be revealed (vs creating distinct transactions
        // for each validator).
        //
        // So instead, we do nothing.

        Ok(self)
    }

    /// Vote with a specific positioned note in the transaction, rather than automatically.
    #[instrument(skip(self, plan))]
    pub fn delegator_vote_precise(&mut self, plan: DelegatorVotePlan) -> &mut Self {
        self.action_list.push(plan);
        self
    }

    #[instrument(skip(self))]
    pub fn lqt_vote(
        &mut self,
        epoch_index: u16,
        incentivized: Denom,
        rewards_recipient: Address,
        notes: &[SpendableNoteRecord],
    ) -> &mut Self {
        let start_position = tct::Position::from((epoch_index, 0, 0));
        for note in notes {
            self.action_list
                .push(ActionLiquidityTournamentVotePlan::new(
                    &mut self.rng,
                    incentivized.clone(),
                    rewards_recipient.clone(),
                    note.note.clone(),
                    note.position,
                    start_position,
                ));
        }
        self
    }

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
    pub fn prioritize_and_filter_spendable_notes(
        &mut self,
        records: Vec<SpendableNoteRecord>,
    ) -> Vec<SpendableNoteRecord> {
        let mut filtered = records
            .into_iter()
            .filter(|record| record.note.amount() > Amount::zero())
            .collect::<Vec<_>>();
        filtered.sort_by(|a, b| {
            // Sort by whether the note was sent to an ephemeral address...
            match (
                a.address_index.is_ephemeral(),
                b.address_index.is_ephemeral(),
            ) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                // ... then by largest amount.
                _ => b.note.amount().cmp(&a.note.amount()),
            }
        });
        filtered
    }

    /// Add spends and change outputs as required to balance the transaction, using the view service
    /// provided to supply the notes and other information.
    pub async fn plan<V: ViewClient + ?Sized>(
        &mut self,
        view: &mut V,
        mut source: AddressIndex,
    ) -> anyhow::Result<TransactionPlan> {
        // Wipe out the randomizer for the provided source, since
        // 1. All randomizers correspond to the same account
        // 2. Using one-time addresses for change addresses is undesirable.
        source.randomizer = [0u8; 12];

        // Compute the change address for this transaction.
        let change_address = if let Some(ref address) = self.change_address {
            address.clone()
        } else {
            view.address_by_index(source).await?.clone()
        };

        // Phase 1, "process all of the user-supplied intents into complete
        // action plans", has already happened using the builder API.
        //
        // Compute an initial fee estimate based on the actions we have so far.
        self.action_list.refresh_fee_and_change(
            &mut self.rng,
            &self
                .gas_prices
                .context("planner instances must call set_gas_prices prior to planning")?,
            &self.fee_tier,
            &change_address,
        );

        // Phase 2: balance the transaction with information from the view service.
        //
        // It's possible that adding spends could increase the gas, increasing
        // the fee amount, and so on, so we add spends iteratively. However, we
        // need to query all the notes we'll use for planning upfront, so we
        // don't accidentally try to use the same one twice.

        let mut notes_by_asset_id = BTreeMap::new();
        for required in self.action_list.balance_with_fee().required() {
            // Find all the notes of this asset in the source account.
            let records: Vec<SpendableNoteRecord> = view
                .notes(NotesRequest {
                    include_spent: false,
                    asset_id: Some(required.asset_id.into()),
                    address_index: Some(source.into()),
                    amount_to_spend: None,
                })
                .await?;
            notes_by_asset_id.insert(
                required.asset_id,
                self.prioritize_and_filter_spendable_notes(records),
            );
        }

        let mut iterations = 0usize;
        let asset_cache = view.assets().await?;

        // Now iterate over the action list's imbalances to balance the transaction.
        while let Some(required) = self.action_list.balance_with_fee().required().next() {
            // Find a single note to spend towards the required balance.
            let note = notes_by_asset_id
                .get_mut(&required.asset_id)
                .expect("we already made a notes request for each required asset")
                .pop()
                .ok_or_else(|| {
                    anyhow!(
                        "ran out of notes to spend while planning transaction, need {}",
                        required.format(&asset_cache)
                    )
                })?;

            // Add a spend for that note to the action list.
            self.action_list
                .push(SpendPlan::new(&mut OsRng, note.note, note.position));

            // Refresh the fee estimate and change outputs.
            self.action_list.refresh_fee_and_change(
                &mut self.rng,
                &self
                    .gas_prices
                    .context("planner instances must call set_gas_prices prior to planning")?,
                &self.fee_tier,
                &change_address,
            );

            iterations = iterations + 1;
            if iterations > 100 {
                return Err(anyhow!("failed to plan transaction after 100 iterations"));
            }
        }

        // Construct the memo plan for the transaction, using user-specified data if it
        // was provided.
        let memo_plan = if self.action_list.requires_memo() {
            let return_address = if let Some(ref address) = self.memo_return_address {
                // Check that this address is actually controlled by the user.
                // We don't have an FVK, so we have to ask the view service.
                anyhow::ensure!(
                    view.index_by_address(address.clone()).await?.is_some(),
                    "return address for memo is not controlled by the user",
                );
                address.clone()
            } else {
                view.address_by_index(source).await?.clone()
            };

            Some(MemoPlan::new(
                &mut self.rng,
                MemoPlaintext::new(return_address, self.memo_text.take().unwrap_or_default())
                    .context("could not create memo plaintext")?,
            ))
        } else {
            None
        };

        // Configure the transaction parameters with the chain ID.
        let app_params = view.app_params().await?;
        let chain_id = app_params.chain_id.clone();
        self.transaction_parameters.chain_id = chain_id.clone();

        // Fetch the FMD parameters that will be used to plan the transaction.
        // (This really should have been considered witness data. Oh well.)
        let fmd_params = view.fmd_parameters().await?;

        let plan = mem::take(&mut self.action_list).into_plan(
            &mut self.rng,
            &fmd_params,
            self.transaction_parameters.clone(),
            memo_plan,
        )?;

        // Reset the planner in case it were reused. We don't want people to do that
        // but otherwise we can't do builder method chaining with &mut self, and forcing
        // the builder to move between calls is annoying for callers who are building up
        // actions programmatically. Except we can't do a normal std::mem::replace here because
        // the generic RNG mucks everything up. So it's just awful.
        self.action_list = Default::default();
        self.gas_prices = Default::default();
        self.fee_tier = Default::default();
        self.transaction_parameters = Default::default();
        self.change_address = None;
        self.memo_text = None;
        self.memo_return_address = None;

        Ok(plan)
    }
}
