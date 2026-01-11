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

use crate::{SpendableNoteRecord, ViewClient, ViewClientComplianceExt};
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

    /// Enrich a transaction plan with compliance details.
    ///
    /// This method adds compliance ciphertexts to spend and output actions:
    /// - For REGULATED assets: encrypts to user's ACK (scannable by registered users)
    /// - For UNREGULATED assets: encrypts to BLACK_HOLE_ACK (nobody can scan)
    /// - For UNREGISTERED assets: returns error (transfers not allowed)
    ///
    /// This ensures all transfers look identical on-chain regardless of regulation status.
    ///
    /// # Limitations
    /// - Multi-spend transactions are not yet supported
    async fn enrich_with_compliance<V: ViewClient + ?Sized>(
        &mut self,
        view: &mut V,
        plan: &mut TransactionPlan,
    ) -> Result<()> {
        use penumbra_sdk_compliance::{ComplianceLeaf, MerklePath, BLACK_HOLE_ACK};
        use penumbra_sdk_keys::keys::AddressComplianceKey;
        use penumbra_sdk_transaction::plan::ActionPlan;

        // First pass: collect indices and check for multi-spend
        let mut spend_indices = Vec::new();
        let mut output_indices = Vec::new();

        for (i, action) in plan.actions.iter().enumerate() {
            match action {
                ActionPlan::Spend(_) => spend_indices.push(i),
                ActionPlan::Output(_) => output_indices.push(i),
                _ => {}
            }
        }

        // Need at least one spend and one output for compliance
        if spend_indices.is_empty() || output_indices.is_empty() {
            return Ok(());
        }

        // Multi-spend not yet supported
        if spend_indices.len() > 1 {
            tracing::debug!("Multi-spend transaction detected, skipping compliance enrichment");
            return Ok(());
        }

        // Get the spend info (we know there's exactly one)
        let spend_idx = spend_indices[0];
        let (asset_id, sender_address) = {
            let ActionPlan::Spend(spend) = &plan.actions[spend_idx] else {
                unreachable!()
            };
            (spend.note.asset_id(), spend.note.address())
        };

        // Fetch the sender's compliance Merkle proofs from the chain
        // This includes paths, positions, anchors, and regulation status
        let sender_proofs = view
            .get_compliance_merkle_proofs(sender_address.clone(), asset_id)
            .await?;

        // Check if asset is registered in the compliance registry
        if !sender_proofs.asset_registered {
            return Err(anyhow!(
                "Asset {:?} is not registered in the compliance registry. \
                Asset issuers must register assets with 'register-asset --regulated' or '--unregulated' before transfers.",
                asset_id
            ));
        }

        let is_regulated = sender_proofs.is_regulated;
        tracing::debug!(
            ?asset_id,
            is_regulated,
            "Asset regulation status from chain"
        );

        // Extract anchors from the proofs (they're the same for all users)
        let compliance_anchor = sender_proofs.compliance_anchor;
        let asset_anchor = sender_proofs.asset_anchor;
        tracing::debug!(
            ?compliance_anchor,
            ?asset_anchor,
            "Fetched compliance anchors from chain"
        );

        // For unregulated assets, we create synthetic leaves with BLACK_HOLE_ACK
        // For regulated assets, we fetch real leaves from the registry
        let black_hole_ack = AddressComplianceKey::new(*BLACK_HOLE_ACK);

        // Create a helper to get the appropriate leaf for an address
        let get_leaf_for_address = |address: &Address, is_regulated: bool| -> ComplianceLeaf {
            if is_regulated {
                // Will be replaced with actual fetch below
                unreachable!("regulated path handled separately")
            } else {
                // For unregulated: synthetic leaf with BLACK_HOLE_ACK
                ComplianceLeaf {
                    address: address.clone(),
                    key: black_hole_ack.clone(),
                    asset_id,
                }
            }
        };

        // Get sender's leaf (real for regulated, synthetic for unregulated)
        let sender_leaf = if is_regulated {
            view.get_compliance_leaf(sender_address.clone(), asset_id)
                .await?
        } else {
            get_leaf_for_address(&sender_address, false)
        };

        // Find the "primary" output (non-change, i.e., different address from sender)
        // to use as counterparty for the spend
        let mut primary_output_idx = None;
        for &idx in &output_indices {
            let ActionPlan::Output(output) = &plan.actions[idx] else {
                continue;
            };
            if output.value.asset_id == asset_id && output.dest_address != sender_address {
                primary_output_idx = Some(idx);
                break;
            }
        }

        // If no primary output found, use the first output with matching asset
        let primary_output_idx = primary_output_idx.unwrap_or_else(|| {
            output_indices
                .iter()
                .copied()
                .find(|&idx| {
                    let ActionPlan::Output(output) = &plan.actions[idx] else {
                        return false;
                    };
                    output.value.asset_id == asset_id
                })
                .unwrap_or(output_indices[0])
        });

        // Get primary recipient info for spend's counterparty
        let primary_recipient_address = {
            let ActionPlan::Output(output) = &plan.actions[primary_output_idx] else {
                unreachable!()
            };
            output.dest_address.clone()
        };

        // Get primary recipient's leaf
        let primary_recipient_leaf = if is_regulated {
            view.get_compliance_leaf(primary_recipient_address.clone(), asset_id)
                .await?
        } else {
            get_leaf_for_address(&primary_recipient_address, false)
        };

        // Set compliance details on the spend
        {
            let ActionPlan::Spend(spend) = &mut plan.actions[spend_idx] else {
                unreachable!()
            };
            spend.set_compliance_details(
                &mut self.rng,
                &sender_leaf.key,
                &sender_address,
                is_regulated,
                &primary_recipient_address,
                primary_recipient_leaf.clone(),
            )?;
            // Set the compliance anchors (always use real anchors from chain)
            spend.compliance_anchor = compliance_anchor;
            spend.asset_anchor = asset_anchor;

            // For regulated: use real Merkle paths from chain
            // For unregulated: use dummy paths (circuit won't verify them)
            if is_regulated {
                spend.compliance_path = sender_proofs.compliance_path.clone();
                spend.compliance_position = sender_proofs.compliance_position;
            } else {
                // Dummy compliance path - circuit skips verification for unregulated
                spend.compliance_path = MerklePath::default();
                spend.compliance_position = 0;
            }
            // Asset path is always real (proves asset's regulation status)
            spend.asset_path = sender_proofs.asset_path.clone();
            spend.asset_position = sender_proofs.asset_position;
        }

        // Get the tx_blinding_nonce from the spend to share with all outputs
        let tx_blinding_nonce = {
            let ActionPlan::Spend(spend) = &plan.actions[spend_idx] else {
                unreachable!()
            };
            spend.tx_blinding_nonce
        };

        // Set compliance details on ALL outputs with matching asset
        for &idx in &output_indices {
            let ActionPlan::Output(output) = &mut plan.actions[idx] else {
                continue;
            };

            // Skip outputs with different asset
            if output.value.asset_id != asset_id {
                continue;
            }

            let recipient_address = output.dest_address.clone();

            // Get recipient's leaf and proofs
            let (recipient_leaf, recipient_compliance_path, recipient_compliance_position) =
                if is_regulated {
                    let proofs = view
                        .get_compliance_merkle_proofs(recipient_address.clone(), asset_id)
                        .await?;
                    let leaf = view
                        .get_compliance_leaf(recipient_address.clone(), asset_id)
                        .await?;
                    (leaf, proofs.compliance_path, proofs.compliance_position)
                } else {
                    // For unregulated: synthetic leaf and dummy path
                    let leaf = get_leaf_for_address(&recipient_address, false);
                    (leaf, MerklePath::default(), 0u64)
                };

            // Set compliance details on this output
            // Counterparty is always the sender (for both destination and change outputs)
            output.set_compliance_details(
                &mut self.rng,
                &recipient_leaf.key,
                is_regulated,
                &sender_address,
                sender_leaf.clone(),
                tx_blinding_nonce,
            )?;
            // Set the compliance anchors (always use real anchors from chain)
            output.compliance_anchor = compliance_anchor;
            output.asset_anchor = asset_anchor;

            // For regulated: use real Merkle paths
            // For unregulated: use dummy paths (circuit skips verification)
            output.compliance_path = recipient_compliance_path;
            output.compliance_position = recipient_compliance_position;
            // Asset path is always real (from sender's proofs - same for all)
            output.asset_path = sender_proofs.asset_path.clone();
            output.asset_position = sender_proofs.asset_position;

            tracing::debug!(
                "Enriched output {} with compliance details (recipient: {:?}, regulated: {})",
                idx,
                recipient_address,
                is_regulated
            );
        }

        tracing::debug!(
            "Successfully enriched transaction with compliance details for asset {:?} (regulated: {}, {} outputs)",
            asset_id,
            is_regulated,
            output_indices.len()
        );

        Ok(())
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

        let mut plan = mem::take(&mut self.action_list).into_plan(
            &mut self.rng,
            &fmd_params,
            self.transaction_parameters.clone(),
            memo_plan,
        )?;

        // Automatically enrich with compliance details if needed
        self.enrich_with_compliance(view, &mut plan).await?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StatusStreamResponse, SwapRecord, TransactionInfo};
    use futures::{FutureExt, Stream};
    use penumbra_sdk_app::params::AppParameters;
    use penumbra_sdk_asset::Value;
    use penumbra_sdk_auction::auction::AuctionId;
    use penumbra_sdk_compliance::TOTAL_WIRE_BYTES;
    use penumbra_sdk_dex::lp::position;
    use penumbra_sdk_fee::GasPrices;
    use penumbra_sdk_proto::view::v1 as pb;
    use penumbra_sdk_sct::Nullifier;
    use penumbra_sdk_shielded_pool::{fmd, note, Note};
    use penumbra_sdk_stake::IdentityKey;
    use penumbra_sdk_transaction::{
        plan::ActionPlan, txhash::TransactionId, AuthorizationData, Transaction, TransactionPlan,
        WitnessData,
    };
    use rand_core::OsRng;
    use std::future::Future;
    use std::pin::Pin;

    /// Mock ViewClient for testing that always returns regulated status
    struct MockRegulatedViewClient;

    impl ViewClient for MockRegulatedViewClient {
        fn compliance_asset_status(
            &mut self,
            _asset_id: asset::Id,
        ) -> Pin<Box<dyn Future<Output = Result<Option<bool>>> + Send + 'static>> {
            async move { Ok(Some(true)) }.boxed()
        }

        // Stub implementations for other required methods
        fn auctions(
            &mut self,
            _: Option<AddressIndex>,
            _: bool,
            _: bool,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<
                            Vec<(
                                AuctionId,
                                SpendableNoteRecord,
                                u64,
                                Option<pbjson_types::Any>,
                                Vec<position::Position>,
                            )>,
                        >,
                    > + Send
                    + 'static,
            >,
        > {
            unimplemented!()
        }
        fn status(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<pb::StatusResponse>> + Send + 'static>> {
            unimplemented!()
        }
        fn status_stream(
            &mut self,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<
                            Pin<
                                Box<
                                    dyn Stream<Item = Result<StatusStreamResponse>>
                                        + Send
                                        + 'static,
                                >,
                            >,
                        >,
                    > + Send
                    + 'static,
            >,
        > {
            unimplemented!()
        }
        fn app_params(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<AppParameters>> + Send + 'static>> {
            unimplemented!()
        }
        fn gas_prices(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<GasPrices>> + Send + 'static>> {
            unimplemented!()
        }
        fn fmd_parameters(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<fmd::Parameters>> + Send + 'static>> {
            unimplemented!()
        }
        fn notes(
            &mut self,
            _: pb::NotesRequest,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<SpendableNoteRecord>>> + Send + 'static>>
        {
            unimplemented!()
        }
        fn notes_for_voting(
            &mut self,
            _: pb::NotesForVotingRequest,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<Vec<(SpendableNoteRecord, IdentityKey)>>>
                    + Send
                    + 'static,
            >,
        > {
            unimplemented!()
        }
        fn balances(
            &mut self,
            _: AddressIndex,
            _: Option<asset::Id>,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<(asset::Id, Amount)>>> + Send + 'static>>
        {
            unimplemented!()
        }
        fn note_by_commitment(
            &mut self,
            _: note::StateCommitment,
        ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>> {
            unimplemented!()
        }
        fn swap_by_commitment(
            &mut self,
            _: penumbra_sdk_tct::StateCommitment,
        ) -> Pin<Box<dyn Future<Output = Result<SwapRecord>> + Send + 'static>> {
            unimplemented!()
        }
        fn nullifier_status(
            &mut self,
            _: Nullifier,
        ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'static>> {
            unimplemented!()
        }
        fn await_nullifier(
            &mut self,
            _: Nullifier,
        ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> {
            unimplemented!()
        }
        fn await_note_by_commitment(
            &mut self,
            _: note::StateCommitment,
        ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>> {
            unimplemented!()
        }
        fn witness(
            &mut self,
            _: &TransactionPlan,
        ) -> Pin<Box<dyn Future<Output = Result<WitnessData>> + Send + 'static>> {
            unimplemented!()
        }
        fn witness_and_build(
            &mut self,
            _: TransactionPlan,
            _: AuthorizationData,
        ) -> Pin<Box<dyn Future<Output = Result<Transaction>> + Send + 'static>> {
            unimplemented!()
        }
        fn assets(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<asset::Cache>> + Send + 'static>> {
            unimplemented!()
        }
        fn owned_position_ids(
            &mut self,
            _: Option<position::State>,
            _: Option<TradingPair>,
            _: Option<AddressIndex>,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<position::Id>>> + Send + 'static>> {
            unimplemented!()
        }
        fn transaction_info_by_hash(
            &mut self,
            _: TransactionId,
        ) -> Pin<Box<dyn Future<Output = Result<TransactionInfo>> + Send + 'static>> {
            unimplemented!()
        }
        fn transaction_info(
            &mut self,
            _: Option<u64>,
            _: Option<u64>,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<TransactionInfo>>> + Send + 'static>> {
            unimplemented!()
        }
        fn broadcast_transaction(
            &mut self,
            _: Transaction,
            _: bool,
        ) -> crate::client::BroadcastStatusStream {
            unimplemented!()
        }
        fn address_by_index(
            &mut self,
            _: AddressIndex,
        ) -> Pin<Box<dyn Future<Output = Result<Address>> + Send + 'static>> {
            unimplemented!()
        }
        fn index_by_address(
            &mut self,
            _: Address,
        ) -> Pin<Box<dyn Future<Output = Result<Option<AddressIndex>>> + Send + 'static>> {
            unimplemented!()
        }
        fn unclaimed_swaps(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<SwapRecord>>> + Send + 'static>> {
            unimplemented!()
        }
        fn lqt_voting_notes(
            &mut self,
            _: u64,
            _: Option<AddressIndex>,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<SpendableNoteRecord>>> + Send + 'static>>
        {
            unimplemented!()
        }
        fn compliance_anchors(
            &mut self,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<(
                            penumbra_sdk_tct::StateCommitment,
                            penumbra_sdk_tct::StateCommitment,
                        )>,
                    > + Send
                    + 'static,
            >,
        > {
            // Return dummy anchors for testing
            async move {
                Ok((
                    penumbra_sdk_tct::StateCommitment(decaf377::Fq::from(0u64)),
                    penumbra_sdk_tct::StateCommitment(decaf377::Fq::from(0u64)),
                ))
            }
            .boxed()
        }
        fn compliance_merkle_proofs(
            &mut self,
            _address: Address,
            _asset_id: asset::Id,
        ) -> Pin<
            Box<dyn Future<Output = Result<pb::ComplianceMerkleProofsResponse>> + Send + 'static>,
        > {
            // Return mock proofs indicating user is registered and asset is regulated
            async move {
                Ok(pb::ComplianceMerkleProofsResponse {
                    user_registered: true,
                    asset_registered: true,
                    is_regulated: true,
                    compliance_path: None, // Empty path for testing
                    compliance_position: 0,
                    asset_path: None,
                    asset_position: 0,
                    compliance_anchor: vec![0u8; 32],
                    asset_anchor: vec![0u8; 32],
                })
            }
            .boxed()
        }
        fn compliance_user_leaf(
            &mut self,
            address: Address,
            asset_id: asset::Id,
        ) -> Pin<Box<dyn Future<Output = Result<pb::ComplianceUserLeafResponse>> + Send + 'static>>
        {
            // Return a mock leaf using demo MCK for consistency with original test behavior
            let leaf = penumbra_sdk_compliance::ComplianceLeaf::new(
                &penumbra_sdk_keys::keys::MasterComplianceKey::demo(),
                address,
                asset_id,
            );
            async move {
                Ok(pb::ComplianceUserLeafResponse {
                    is_registered: true,
                    leaf: Some(pb::ComplianceLeaf {
                        address: Some(leaf.address.into()),
                        key: Some(pb::ComplianceViewingKey {
                            inner: leaf.key.0.vartime_compress().0.to_vec(),
                        }),
                        asset_id: Some(leaf.asset_id.into()),
                    }),
                })
            }
            .boxed()
        }
    }

    #[tokio::test]
    async fn test_planner_auto_enriches_compliance() {
        let mut rng = OsRng;

        // Create sender and recipient addresses
        let sender_address = Address::dummy(&mut rng);
        let recipient_address = Address::dummy(&mut rng);

        // Create a test asset
        let asset_id = asset::Id(decaf377::Fq::from(999u64));
        let value = Value {
            amount: 100u64.into(),
            asset_id,
        };

        // Create a note for the spend
        let note = Note::from_parts(
            sender_address.clone(),
            value,
            penumbra_sdk_shielded_pool::Rseed::generate(&mut rng),
        )
        .expect("valid note");

        // Create a transaction plan with spend and output
        let spend_plan = SpendPlan::new(&mut rng, note, 0u64.into());
        let output_plan = OutputPlan::new(&mut rng, value, recipient_address.clone());

        let mut plan = TransactionPlan {
            actions: vec![
                ActionPlan::Spend(spend_plan),
                ActionPlan::Output(output_plan),
            ],
            transaction_parameters: TransactionParameters::default(),
            detection_data: None,
            memo: None,
        };

        // Create a planner (we won't use it to plan, just to call enrich_with_compliance)
        let mut planner = Planner::new(OsRng);
        let mut mock_view = MockRegulatedViewClient;

        // Call enrich_with_compliance directly
        let result = planner
            .enrich_with_compliance(&mut mock_view, &mut plan)
            .await;
        assert!(
            result.is_ok(),
            "Compliance enrichment should succeed: {:?}",
            result.err()
        );

        // Verify that compliance details were set on both spend and output
        for action in &plan.actions {
            match action {
                ActionPlan::Spend(sp) => {
                    assert!(
                        sp.compliance_leaf.is_some(),
                        "Spend should have compliance leaf set"
                    );
                    assert!(
                        sp.compliance_ephemeral_secret.is_some(),
                        "Spend should have ephemeral secret"
                    );
                    assert!(
                        sp.counterparty_leaf.is_some(),
                        "Spend should have counterparty leaf"
                    );
                    assert_eq!(sp.is_regulated, true, "Asset should be marked as regulated");
                }
                ActionPlan::Output(op) => {
                    assert!(
                        op.compliance_leaf.is_some(),
                        "Output should have compliance leaf set"
                    );
                    assert!(
                        op.compliance_ephemeral_secret.is_some(),
                        "Output should have ephemeral secret"
                    );
                    assert!(
                        op.counterparty_leaf.is_some(),
                        "Output should have counterparty leaf"
                    );
                    assert_eq!(op.is_regulated, true, "Asset should be marked as regulated");
                }
                _ => {}
            }
        }
    }

    /// Mock ViewClient for testing that always returns unregulated status
    struct MockUnregulatedViewClient;

    impl ViewClient for MockUnregulatedViewClient {
        fn compliance_asset_status(
            &mut self,
            _asset_id: asset::Id,
        ) -> Pin<Box<dyn Future<Output = Result<Option<bool>>> + Send + 'static>> {
            async move { Ok(Some(false)) }.boxed()
        }

        // Stub implementations (same as above)
        fn auctions(
            &mut self,
            _: Option<AddressIndex>,
            _: bool,
            _: bool,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<
                            Vec<(
                                AuctionId,
                                SpendableNoteRecord,
                                u64,
                                Option<pbjson_types::Any>,
                                Vec<position::Position>,
                            )>,
                        >,
                    > + Send
                    + 'static,
            >,
        > {
            unimplemented!()
        }
        fn status(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<pb::StatusResponse>> + Send + 'static>> {
            unimplemented!()
        }
        fn status_stream(
            &mut self,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<
                            Pin<
                                Box<
                                    dyn Stream<Item = Result<StatusStreamResponse>>
                                        + Send
                                        + 'static,
                                >,
                            >,
                        >,
                    > + Send
                    + 'static,
            >,
        > {
            unimplemented!()
        }
        fn app_params(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<AppParameters>> + Send + 'static>> {
            unimplemented!()
        }
        fn gas_prices(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<GasPrices>> + Send + 'static>> {
            unimplemented!()
        }
        fn fmd_parameters(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<fmd::Parameters>> + Send + 'static>> {
            unimplemented!()
        }
        fn notes(
            &mut self,
            _: pb::NotesRequest,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<SpendableNoteRecord>>> + Send + 'static>>
        {
            unimplemented!()
        }
        fn notes_for_voting(
            &mut self,
            _: pb::NotesForVotingRequest,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<Vec<(SpendableNoteRecord, IdentityKey)>>>
                    + Send
                    + 'static,
            >,
        > {
            unimplemented!()
        }
        fn balances(
            &mut self,
            _: AddressIndex,
            _: Option<asset::Id>,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<(asset::Id, Amount)>>> + Send + 'static>>
        {
            unimplemented!()
        }
        fn note_by_commitment(
            &mut self,
            _: note::StateCommitment,
        ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>> {
            unimplemented!()
        }
        fn swap_by_commitment(
            &mut self,
            _: penumbra_sdk_tct::StateCommitment,
        ) -> Pin<Box<dyn Future<Output = Result<SwapRecord>> + Send + 'static>> {
            unimplemented!()
        }
        fn nullifier_status(
            &mut self,
            _: Nullifier,
        ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'static>> {
            unimplemented!()
        }
        fn await_nullifier(
            &mut self,
            _: Nullifier,
        ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> {
            unimplemented!()
        }
        fn await_note_by_commitment(
            &mut self,
            _: note::StateCommitment,
        ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>> {
            unimplemented!()
        }
        fn witness(
            &mut self,
            _: &TransactionPlan,
        ) -> Pin<Box<dyn Future<Output = Result<WitnessData>> + Send + 'static>> {
            unimplemented!()
        }
        fn witness_and_build(
            &mut self,
            _: TransactionPlan,
            _: AuthorizationData,
        ) -> Pin<Box<dyn Future<Output = Result<Transaction>> + Send + 'static>> {
            unimplemented!()
        }
        fn assets(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<asset::Cache>> + Send + 'static>> {
            unimplemented!()
        }
        fn owned_position_ids(
            &mut self,
            _: Option<position::State>,
            _: Option<TradingPair>,
            _: Option<AddressIndex>,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<position::Id>>> + Send + 'static>> {
            unimplemented!()
        }
        fn transaction_info_by_hash(
            &mut self,
            _: TransactionId,
        ) -> Pin<Box<dyn Future<Output = Result<TransactionInfo>> + Send + 'static>> {
            unimplemented!()
        }
        fn transaction_info(
            &mut self,
            _: Option<u64>,
            _: Option<u64>,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<TransactionInfo>>> + Send + 'static>> {
            unimplemented!()
        }
        fn broadcast_transaction(
            &mut self,
            _: Transaction,
            _: bool,
        ) -> crate::client::BroadcastStatusStream {
            unimplemented!()
        }
        fn address_by_index(
            &mut self,
            _: AddressIndex,
        ) -> Pin<Box<dyn Future<Output = Result<Address>> + Send + 'static>> {
            unimplemented!()
        }
        fn index_by_address(
            &mut self,
            _: Address,
        ) -> Pin<Box<dyn Future<Output = Result<Option<AddressIndex>>> + Send + 'static>> {
            unimplemented!()
        }
        fn unclaimed_swaps(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<SwapRecord>>> + Send + 'static>> {
            unimplemented!()
        }
        fn lqt_voting_notes(
            &mut self,
            _: u64,
            _: Option<AddressIndex>,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<SpendableNoteRecord>>> + Send + 'static>>
        {
            unimplemented!()
        }
        fn compliance_anchors(
            &mut self,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<(
                            penumbra_sdk_tct::StateCommitment,
                            penumbra_sdk_tct::StateCommitment,
                        )>,
                    > + Send
                    + 'static,
            >,
        > {
            // Return dummy anchors for testing
            async move {
                Ok((
                    penumbra_sdk_tct::StateCommitment(decaf377::Fq::from(0u64)),
                    penumbra_sdk_tct::StateCommitment(decaf377::Fq::from(0u64)),
                ))
            }
            .boxed()
        }
        fn compliance_merkle_proofs(
            &mut self,
            _address: Address,
            _asset_id: asset::Id,
        ) -> Pin<
            Box<dyn Future<Output = Result<pb::ComplianceMerkleProofsResponse>> + Send + 'static>,
        > {
            // Return mock proofs indicating asset is NOT regulated
            async move {
                Ok(pb::ComplianceMerkleProofsResponse {
                    user_registered: false,
                    asset_registered: true,
                    is_regulated: false, // Key: unregulated
                    compliance_path: None,
                    compliance_position: 0,
                    asset_path: None,
                    asset_position: 0,
                    compliance_anchor: vec![0u8; 32],
                    asset_anchor: vec![0u8; 32],
                })
            }
            .boxed()
        }
        fn compliance_user_leaf(
            &mut self,
            _address: Address,
            _asset_id: asset::Id,
        ) -> Pin<Box<dyn Future<Output = Result<pb::ComplianceUserLeafResponse>> + Send + 'static>>
        {
            // Return not registered for unregulated asset
            async move {
                Ok(pb::ComplianceUserLeafResponse {
                    is_registered: false,
                    leaf: None,
                })
            }
            .boxed()
        }
    }

    #[tokio::test]
    async fn test_planner_skips_compliance_for_unregulated_assets() {
        let mut rng = OsRng;

        // Create sender and recipient addresses
        let sender_address = Address::dummy(&mut rng);
        let recipient_address = Address::dummy(&mut rng);

        // Create a test asset (unregulated)
        let asset_id = asset::Id(decaf377::Fq::from(888u64));
        let value = Value {
            amount: 100u64.into(),
            asset_id,
        };

        // Create a note for the spend
        let note = Note::from_parts(
            sender_address.clone(),
            value,
            penumbra_sdk_shielded_pool::Rseed::generate(&mut rng),
        )
        .expect("valid note");

        // Create a transaction plan with spend and output
        let spend_plan = SpendPlan::new(&mut rng, note, 0u64.into());
        let output_plan = OutputPlan::new(&mut rng, value, recipient_address.clone());

        let mut plan = TransactionPlan {
            actions: vec![
                ActionPlan::Spend(spend_plan),
                ActionPlan::Output(output_plan),
            ],
            transaction_parameters: TransactionParameters::default(),
            detection_data: None,
            memo: None,
        };

        // Create a planner and mock view that returns unregulated status
        let mut planner = Planner::new(OsRng);
        let mut mock_view = MockUnregulatedViewClient;

        // Call enrich_with_compliance
        let result = planner
            .enrich_with_compliance(&mut mock_view, &mut plan)
            .await;
        assert!(
            result.is_ok(),
            "Compliance enrichment should succeed even for unregulated assets"
        );

        // Verify that compliance details ARE set but with unregulated flag
        // (unregulated assets use BLACK_HOLE_ACK, making ciphertexts indistinguishable)
        for action in &plan.actions {
            match action {
                ActionPlan::Spend(sp) => {
                    assert!(
                        sp.compliance_leaf.is_some(),
                        "Spend should have compliance leaf (with BLACK_HOLE_ACK for unregulated)"
                    );
                    assert_eq!(
                        sp.is_regulated, false,
                        "Asset should be marked as unregulated"
                    );
                }
                ActionPlan::Output(op) => {
                    assert!(
                        op.compliance_leaf.is_some(),
                        "Output should have compliance leaf (with BLACK_HOLE_ACK for unregulated)"
                    );
                    assert_eq!(
                        op.is_regulated, false,
                        "Asset should be marked as unregulated"
                    );
                }
                _ => {}
            }
        }
    }

    #[tokio::test]
    async fn test_planner_enriches_multi_output_with_change() {
        let mut rng = OsRng;

        // Create sender and recipient addresses
        let sender_address = Address::dummy(&mut rng);
        let recipient_address = Address::dummy(&mut rng);

        // Create a test asset (regulated)
        let asset_id = asset::Id(decaf377::Fq::from(777u64));

        // Value for the main transfer
        let transfer_value = Value {
            amount: 100u64.into(),
            asset_id,
        };

        // Value for the change (back to sender)
        let change_value = Value {
            amount: 50u64.into(),
            asset_id,
        };

        // Create a note for the spend (larger than transfer to have change)
        let spend_value = Value {
            amount: 150u64.into(),
            asset_id,
        };
        let note = Note::from_parts(
            sender_address.clone(),
            spend_value,
            penumbra_sdk_shielded_pool::Rseed::generate(&mut rng),
        )
        .expect("valid note");

        // Create a transaction plan with spend, output to recipient, and change output to sender
        let spend_plan = SpendPlan::new(&mut rng, note, 0u64.into());
        let output_to_recipient =
            OutputPlan::new(&mut rng, transfer_value, recipient_address.clone());
        let change_output = OutputPlan::new(&mut rng, change_value, sender_address.clone());

        let mut plan = TransactionPlan {
            actions: vec![
                ActionPlan::Spend(spend_plan),
                ActionPlan::Output(output_to_recipient),
                ActionPlan::Output(change_output),
            ],
            transaction_parameters: TransactionParameters::default(),
            detection_data: None,
            memo: None,
        };

        // Create a planner and mock view that returns regulated status
        let mut planner = Planner::new(OsRng);
        let mut mock_view = MockRegulatedViewClient;

        // Call enrich_with_compliance
        let result = planner
            .enrich_with_compliance(&mut mock_view, &mut plan)
            .await;
        assert!(
            result.is_ok(),
            "Compliance enrichment should succeed for multi-output: {:?}",
            result.err()
        );

        // Verify that compliance details were set on spend and BOTH outputs
        let mut spend_enriched = false;
        let mut outputs_enriched = 0;

        for action in &plan.actions {
            match action {
                ActionPlan::Spend(sp) => {
                    assert!(
                        sp.compliance_leaf.is_some(),
                        "Spend should have compliance leaf set"
                    );
                    assert!(
                        sp.compliance_ephemeral_secret.is_some(),
                        "Spend should have ephemeral secret"
                    );
                    assert!(
                        sp.counterparty_leaf.is_some(),
                        "Spend should have counterparty leaf"
                    );
                    assert_eq!(sp.is_regulated, true, "Asset should be marked as regulated");
                    spend_enriched = true;
                }
                ActionPlan::Output(op) => {
                    assert!(
                        op.compliance_leaf.is_some(),
                        "Output should have compliance leaf set"
                    );
                    assert!(
                        op.compliance_ephemeral_secret.is_some(),
                        "Output should have ephemeral secret"
                    );
                    assert!(
                        op.counterparty_leaf.is_some(),
                        "Output should have counterparty leaf"
                    );
                    assert_eq!(op.is_regulated, true, "Asset should be marked as regulated");
                    // Verify ciphertext is TOTAL_WIRE_BYTES (not placeholder)
                    assert_eq!(
                        op.compliance_ciphertext.len(),
                        TOTAL_WIRE_BYTES,
                        "Compliance ciphertext should be TOTAL_WIRE_BYTES"
                    );
                    outputs_enriched += 1;
                }
                _ => {}
            }
        }

        assert!(spend_enriched, "Spend should be enriched");
        assert_eq!(
            outputs_enriched, 2,
            "Both outputs (recipient + change) should be enriched"
        );
    }
}
