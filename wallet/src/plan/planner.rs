use std::{
    fmt::{self, Debug, Formatter},
    mem,
};

use anyhow::{anyhow, Result};

use penumbra_component::stake::{rate::RateData, validator};
use penumbra_crypto::{
    asset::Amount,
    asset::Denom,
    dex::{swap::SwapPlaintext, BatchSwapOutputData, TradingPair},
    keys::AddressIndex,
    rdsa::{SpendAuth, VerificationKey},
    transaction::Fee,
    Address, FieldExt, Fr, FullViewingKey, Note, Value,
};
use penumbra_proto::view::v1alpha1::NotesRequest;
use penumbra_tct as tct;
use penumbra_transaction::{
    action::{Proposal, ProposalSubmit, ProposalWithdrawBody, ValidatorVote},
    plan::{
        ActionPlan, MemoPlan, OutputPlan, ProposalWithdrawPlan, SpendPlan, SwapClaimPlan, SwapPlan,
        TransactionPlan, UndelegateClaimPlan,
    },
};
use penumbra_view::ViewClient;
use rand::{CryptoRng, RngCore};
use tracing::instrument;

use penumbra_crypto::Balance;

/// A planner for a [`TransactionPlan`] that can fill in the required spends and change outputs upon
/// finalization to make a transaction balance.
pub struct Planner<R: RngCore + CryptoRng> {
    rng: R,
    balance: Balance,
    plan: TransactionPlan,
    proposal_submits: Vec<Proposal>,
    proposal_withdraws: Vec<(Address, ProposalWithdrawBody)>,
    // IMPORTANT: if you add more fields here, make sure to clear them when the planner is finished
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
            plan: TransactionPlan::default(),
            proposal_submits: Vec::new(),
            proposal_withdraws: Vec::new(),
        }
    }

    /// Get the current transaction balance of the planner.
    pub fn balance(&self) -> &Balance {
        &self.balance
    }

    /// Set the expiry height for the transaction plan.
    #[instrument(skip(self))]
    pub fn expiry_height(&mut self, expiry_height: u64) -> &mut Self {
        self.plan.expiry_height = expiry_height;
        self
    }

    /// Set a memo for this transaction plan.
    ///
    /// Errors if the memo is too long.
    #[instrument(skip(self))]
    pub fn memo(&mut self, memo: String) -> anyhow::Result<&mut Self> {
        self.plan.memo_plan = Some(MemoPlan::new(&mut self.rng, memo)?);
        Ok(self)
    }

    /// Add a fee to the transaction plan.
    ///
    /// This function should be called once.
    #[instrument(skip(self))]
    pub fn fee(&mut self, fee: Fee) -> &mut Self {
        self.balance += fee.0;
        self.plan.fee = fee;
        self
    }

    /// Spend a specific positioned note in the transaction.
    ///
    /// If you don't use this method to specify spends, they will be filled in automatically from
    /// the view service when the plan is [`finish`](Builder::finish)ed.
    #[instrument(skip(self))]
    pub fn spend(&mut self, note: Note, position: tct::Position) -> &mut Self {
        let spend = SpendPlan::new(&mut self.rng, note, position).into();
        self.action(spend);
        self
    }

    /// Perform a swap claim based on an input swap NFT with a pre-paid fee.
    #[instrument(skip(self))]
    pub fn swap_claim(
        &mut self,
        swap_plaintext: SwapPlaintext,
        swap_nft_note: Note,
        swap_nft_position: tct::Position,
        epoch_duration: u64,
        output_data: BatchSwapOutputData,
    ) -> &mut Self {
        // Add a `SwapClaimPlan` action:
        let swap_claim = SwapClaimPlan::new(
            &mut self.rng,
            swap_plaintext,
            swap_nft_note,
            swap_nft_position,
            epoch_duration,
            output_data,
        )
        .into();

        // Nothing needs to be spent, since the fee is pre-paid and the
        // swap NFT will be automatically consumed when the SwapClaim action
        // is processed by the validators.
        self.action(swap_claim);
        self
    }

    /// Perform a swap based on input notes in the transaction.
    #[instrument(skip(self))]
    pub fn swap(
        &mut self,
        input_value: Value,
        into_denom: Denom,
        swap_claim_fee: Fee,
        claim_address: Address,
    ) -> Result<&mut Self> {
        // Determine the canonical order for the assets being swapped.
        // This will determine whether the input amount is assigned to delta_1 or delta_2.
        let trading_pair =
            TradingPair::canonical_order_for((input_value.asset_id, into_denom.id()))?;

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
            return Err(anyhow!("No input value for swap"));
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
    pub fn delegate(&mut self, unbonded_amount: u64, rate_data: RateData) -> &mut Self {
        let delegation = rate_data.build_delegate(unbonded_amount).into();
        self.action(delegation);
        self
    }

    /// Add an undelegation to this transaction.
    ///
    /// TODO: can we put the chain parameters into the planner at the start, so we can compute end_epoch_index?
    #[instrument(skip(self))]
    pub fn undelegate(
        &mut self,
        delegation_amount: Amount,
        rate_data: RateData,
        end_epoch_index: u64,
    ) -> &mut Self {
        let undelegation = rate_data
            .build_undelegate(delegation_amount, end_epoch_index)
            .into();
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
        self.action(ActionPlan::ValidatorDefinition(new_validator.into()));
        self
    }

    /// Submit a new governance proposal in this transaction.
    #[instrument(skip(self))]
    pub fn proposal_submit(&mut self, proposal: Proposal) -> &mut Self {
        self.proposal_submits.push(proposal);
        self
    }

    /// Withdraw a governance proposal in this transaction.
    #[instrument(skip(self))]
    pub fn proposal_withdraw(
        &mut self,
        proposal_id: u64,
        deposit_refund_address: Address,
        reason: String,
    ) -> &mut Self {
        self.proposal_withdraws.push((
            deposit_refund_address,
            ProposalWithdrawBody {
                proposal: proposal_id,
                reason,
            },
        ));
        self
    }

    /// Cast a validator vote in this transaction.
    #[instrument(skip(self))]
    pub fn validator_vote(&mut self, vote: ValidatorVote) -> &mut Self {
        self.action(ActionPlan::ValidatorVote(vote));
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
    #[instrument(skip(self, view, fvk))]
    pub async fn plan<V: ViewClient>(
        &mut self,
        view: &mut V,
        fvk: &FullViewingKey,
        source: Option<AddressIndex>,
    ) -> anyhow::Result<TransactionPlan> {
        tracing::debug!(plan = ?self.plan, balance = ?self.balance, "finalizing transaction");

        // Fill in the chain id based on the view service
        let chain_params = view.chain_params().await?;
        self.plan.chain_id = chain_params.chain_id;

        // Proposals aren't actually turned into action plans until now, because we need the view
        // service to fill in the details. Now we have the chain parameters and the FVK, so we can
        // automatically fill in the rest of the action plan without asking the user for anything:
        for proposal in mem::take(&mut self.proposal_submits) {
            let (deposit_refund_address, withdraw_proposal_key) =
                self.proposal_address_and_withdraw_key(fvk);

            self.action(
                ProposalSubmit {
                    proposal,
                    deposit_amount: chain_params.proposal_deposit_amount,
                    deposit_refund_address,
                    withdraw_proposal_key,
                }
                .into(),
            );
        }

        // Similarly, proposal withdrawals need the FVK to convert the address into the original
        // randomizer, so we delay adding it to the transaction plan until now
        for (address, body) in mem::take(&mut self.proposal_withdraws) {
            let randomizer = self.proposal_withdraw_randomizer(fvk, &address);
            self.action(ProposalWithdrawPlan { body, randomizer }.into());
        }

        // Get all notes required to fulfill needed spends
        let mut spends = Vec::new();
        for Value { amount, asset_id } in self.balance.required() {
            spends.extend(
                view.notes(NotesRequest {
                    account_id: Some(fvk.hash().into()),
                    asset_id: Some(asset_id.into()),
                    address_index: source.map(Into::into),
                    amount_to_spend: amount.into(),
                    include_spent: false,
                    ..Default::default()
                })
                .await?,
            );
        }

        // Add the required spends to the planner
        for record in spends {
            self.spend(record.note, record.position);
        }

        // For any remaining provided balance, make a single change note for each
        let self_address = fvk
            .incoming()
            .payment_address(source.unwrap_or(AddressIndex::Numeric(0)))
            .0;

        for value in self.balance.provided().collect::<Vec<_>>() {
            self.output(value, self_address);
        }

        // If there are outputs, we check that a memo has been added. If not, we add a default memo.
        if self.plan.num_outputs() > 0 && self.plan.memo_plan.is_none() {
            self.memo(String::new())
                .expect("empty string is a valid memo");
        } else if self.plan.num_outputs() == 0 && self.plan.memo_plan.is_some() {
            anyhow::bail!("if no outputs, no memo should be added");
        }

        // Add clue plans for `Output`s.
        let fmd_params = view.fmd_parameters().await?;
        let precision_bits = fmd_params.precision_bits;
        self.plan
            .add_all_clue_plans(&mut self.rng, precision_bits.into());

        // Now the transaction should be fully balanced, unless we didn't have enough to spend
        if !self.balance.is_zero() {
            anyhow::bail!(
                "balance is non-zero after attempting to balance transaction: {:?}",
                self.balance
            );
        }

        tracing::debug!(plan = ?self.plan, "finished balancing transaction");

        // Clear the planner and pull out the plan to return
        self.balance = Balance::zero();
        let plan = mem::take(&mut self.plan);

        Ok(plan)
    }

    /// Get a random address/withdraw key pair for proposals.
    fn proposal_address_and_withdraw_key(
        &mut self,
        fvk: &FullViewingKey,
    ) -> (Address, VerificationKey<SpendAuth>) {
        // The deposit refund address should be an ephemeral address
        let deposit_refund_address = fvk.incoming().ephemeral_address(&mut self.rng).0;

        // The proposal withdraw verification key is the spend auth verification key randomized by the
        // deposit refund address's address index
        let withdraw_proposal_key = {
            // Use the fvk to get the original address index of the diversifier
            let deposit_refund_address_index = fvk
                .incoming()
                .index_for_diversifier(deposit_refund_address.diversifier());

            // Convert this to a vector
            let mut deposit_refund_address_index_bytes =
                deposit_refund_address_index.to_bytes().to_vec();

            // Pad it with zeros to be 32 bytes long (the size expected by a randomizer)
            deposit_refund_address_index_bytes.extend([0; 16]);

            // Convert it back to exactly 32 bytes
            let deposit_refund_address_index_bytes = deposit_refund_address_index_bytes
                .try_into()
                .expect("exactly 32 bytes");

            // Get the scalar `Fr` element derived from these bytes
            let withdraw_proposal_key_randomizer =
                Fr::from_bytes(deposit_refund_address_index_bytes)
                    .expect("bytes are within range for `Fr`");

            // Randomize the spend verification key for the fvk using this randomizer
            fvk.spend_verification_key()
                .randomize(&withdraw_proposal_key_randomizer)
        };

        (deposit_refund_address, withdraw_proposal_key)
    }

    /// Get the randomizer from an address using the FVK.
    fn proposal_withdraw_randomizer(&self, fvk: &FullViewingKey, address: &Address) -> Fr {
        // Use the fvk to get the original address index of the diversifier
        let deposit_refund_address_index =
            fvk.incoming().index_for_diversifier(address.diversifier());

        // Convert this to a vector
        let mut deposit_refund_address_index_bytes =
            deposit_refund_address_index.to_bytes().to_vec();
        // Pad it with zeros to be 32 bytes long (the size expected by a randomizer)
        deposit_refund_address_index_bytes.extend([0; 16]);
        // Convert it back to exactly 32 bytes
        let deposit_refund_address_index_bytes = deposit_refund_address_index_bytes
            .try_into()
            .expect("exactly 32 bytes");

        // Get the scalar `Fr` element derived from these bytes
        Fr::from_bytes(deposit_refund_address_index_bytes).expect("bytes are within range for `Fr`")
    }
}
