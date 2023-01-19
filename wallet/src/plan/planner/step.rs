//! A step-by-step externally driven interface to the [`Planner`], not necessary if a [`ViewClient`]
//! can be furnished. Prefer [`Planner::plan`] if this is so.
//!
//! However, in some cases, a [`ViewClient`] may not be available, such as in the case of WASM
//! compilation. While [`Planner::plan`] *drives* the [`ViewClient`], this module inverts control so
//! that an external piece of code can drive iteration through the queries necessary to finish
//! planning a transaction. This is necessary for WASM compilation, where we cannot query a database
//! from within the WASM sandbox.
//!
//! For a canonical example of how to interact with this interface, see the implementation of the
//! [`Planner::plan`] method.

use penumbra_chain::params::FmdParameters;

use super::*;

/// An intermediate result of a step-by-step externally driven querying of some view service.
pub enum Step<'a, R: RngCore + CryptoRng> {
    /// No more queries are needed, the transaction plan is finished.
    Finished(TransactionPlan),
    /// A request must be fulfilled to continue building the transaction plan.
    ///
    /// When the request is fulfilled, call [`Respond::with`] with the response to the request.
    Request {
        request: NotesRequest,
        respond: Respond<'a, R>,
    },
}

/// A continuation of a step-by-step externally driven querying of some view service.
///
/// To resume the continuation, call [`Respond::with`] with the response to the request.
pub struct Respond<'a, R: RngCore + CryptoRng> {
    planner: &'a mut Planner<R>,
    fvk: &'a FullViewingKey,
    fmd_params: &'a FmdParameters,
    source: Option<AddressIndex>,
    remaining_requests: Vec<Value>,
    provided_records: Vec<SpendableNoteRecord>,
}

impl<R: RngCore + CryptoRng> Planner<R> {
    pub(super) fn start<'a>(
        &'a mut self,
        chain_params: &'a ChainParameters,
        fmd_params: &'a FmdParameters,
        fvk: &'a FullViewingKey,
        source: Option<AddressIndex>,
    ) -> anyhow::Result<Step<'a, R>> {
        self.plan.chain_id = chain_params.chain_id.clone();

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

        // Here are all the requests we need to make in order to fulfill this plan
        let remaining_requests = self.balance.required().collect();

        let respond = Respond {
            fvk,
            fmd_params,
            source,
            remaining_requests,
            provided_records: Vec::new(),
            planner: self,
        };

        respond.step()
    }
}

impl<'a, R: RngCore + CryptoRng> Respond<'a, R> {
    /// Make a request for notes if one is needed, or else finalize the transaction plan.
    fn step(mut self) -> anyhow::Result<Step<'a, R>> {
        if let Some(value) = self.remaining_requests.pop() {
            let request = NotesRequest {
                account_id: Some(self.fvk.hash().into()),
                asset_id: Some(value.asset_id.into()),
                address_index: self.source.map(Into::into),
                amount_to_spend: value.amount.into(),
                include_spent: false,
                ..Default::default()
            };

            return Ok(Step::Request {
                request,
                respond: self,
            });
        }

        // Once all requests processed, we can add the required spends to the planner
        for record in self.provided_records {
            self.planner.spend(record.note, record.position);
        }

        // Now we can finalize the transaction plan, because no more interaction is needed
        self.planner
            .finalize(self.fmd_params, self.fvk, self.source)
            .map(Step::Finished)
    }

    /// Supply a response to a request for notes, finalizing the transaction plan if this is the
    /// last request.
    pub fn with(mut self, record: Vec<SpendableNoteRecord>) -> anyhow::Result<Step<'a, R>> {
        self.provided_records.extend(record);
        self.step()
    }
}

impl<R: RngCore + CryptoRng> Planner<R> {
    /// Finalize the transaction plan, adding change outputs and a memo if needed, as well as clue
    /// parameters, and resetting the contained planner to the initial state.
    fn finalize(
        &mut self,
        fmd_params: &FmdParameters,
        fvk: &FullViewingKey,
        source: Option<AddressIndex>,
    ) -> anyhow::Result<TransactionPlan> {
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
}
