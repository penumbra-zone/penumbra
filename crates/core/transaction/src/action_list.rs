use anyhow::Result;
use std::collections::BTreeMap;

use crate::plan::MemoPlan;
use crate::{gas::GasCost, TransactionParameters};
use crate::{ActionPlan, TransactionPlan};
use penumbra_sdk_asset::{asset, Balance};
use penumbra_sdk_fee::{Fee, FeeTier, Gas, GasPrices};
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::Amount;
use penumbra_sdk_shielded_pool::{fmd, OutputPlan};
use rand_core::{CryptoRng, RngCore};

/// A list of planned actions to be turned into a TransactionPlan.
///
/// A transaction is a bundle of actions plus auxiliary data (like a memo). A
/// transaction plan is a bundle of action plans plus plans for the auxiliary
/// data (like a memo plan).  The [`ActionList`] is just the list of actions,
/// which is useful for building up a [`TransactionPlan`].
#[derive(Debug, Default, Clone)]
pub struct ActionList {
    // A list of the user-specified outputs.
    actions: Vec<ActionPlan>,
    // These are tracked separately for convenience when adjusting change.
    change_outputs: BTreeMap<asset::Id, OutputPlan>,
    // The fee is tracked as part of the ActionList so it can be adjusted
    // internally to handle special cases like swap claims.
    fee: Fee,
}

impl ActionList {
    /// Returns an immutable reference to a list of action plans.
    pub fn actions(&self) -> &Vec<ActionPlan> {
        &self.actions
    }

    /// Returns an immutable reference to a map of change outputs.
    pub fn change_outputs(&self) -> &BTreeMap<asset::Id, OutputPlan> {
        &self.change_outputs
    }

    /// Returns an immutable reference to the fee.
    pub fn fee(&self) -> &Fee {
        &self.fee
    }

    /// Returns true if the resulting transaction would require a memo.
    pub fn requires_memo(&self) -> bool {
        let has_change_outputs = !self.change_outputs.is_empty();
        let has_other_outputs = self
            .actions
            .iter()
            .any(|action| matches!(action, ActionPlan::Output(_)));

        has_change_outputs || has_other_outputs
    }

    /// Convert this list of actions into a [`TransactionPlan`].
    pub fn into_plan<R: RngCore + CryptoRng>(
        self,
        rng: R,
        fmd_params: &fmd::Parameters,
        mut transaction_parameters: TransactionParameters,
        memo_plan: Option<MemoPlan>,
    ) -> Result<TransactionPlan> {
        transaction_parameters.fee = self.fee;

        let mut plan = TransactionPlan {
            actions: self
                .actions
                .into_iter()
                .chain(self.change_outputs.into_values().map(Into::into))
                .collect(),
            transaction_parameters,
            memo: memo_plan,
            detection_data: None,
        };
        plan.populate_detection_data(rng, fmd_params.precision);

        // Implement a canonical ordering to the actions within the transaction
        // plan to reduce client distinguishability.
        plan.sort_actions();

        Ok(plan)
    }

    /// Push a new action onto this list.
    pub fn push<A: Into<ActionPlan>>(&mut self, action: A) {
        let plan = action.into();

        // Special case: if the plan is a `SwapClaimPlan`, adjust the fee to include the
        // prepaid fee contributed by the swap claim. This helps ensure that the value
        // released by the swap claim is used to pay the fee, rather than generating change.
        if let ActionPlan::SwapClaim(claim) = &plan {
            let claim_fee = claim.swap_plaintext.claim_fee;
            if self.fee.amount() == Amount::zero() {
                // If the fee is currently zero, set it to the claim fee,
                // regardless of fee token, i.e., set the fee token to match
                // the swap claim.
                self.fee = claim_fee;
            } else if self.fee.asset_matches(&claim_fee) {
                // Otherwise, if the fee token matches, accumulate the amount
                // released by the swap claim into the fee, rather than letting it
                // be handled as change.
                self.fee.0.amount += claim_fee.amount();
            } else {
                // In this situation, the fee has been manually set to a
                // different token than was released by the swap claim. So we
                // can't accumulate the swap claim fee into it, and it will
                // produce change instead.
            }
        }

        self.actions.push(plan);
    }

    /// Compute the gas used by a transaction comprised of the actions in this list.
    ///
    /// Because Penumbra transactions have static gas costs, and gas use is linear in the actions,
    /// this is an exact computation.
    fn gas_cost(&self) -> Gas {
        let mut gas = Gas::zero();
        for action in &self.actions {
            // TODO missing AddAssign
            gas = gas + action.gas_cost();
        }
        for action in self.change_outputs.values() {
            // TODO missing AddAssign
            // TODO missing GasCost impl on OutputPlan
            gas = gas + ActionPlan::from(action.clone()).gas_cost();
        }

        gas
    }

    /// Use the provided gas prices and fee tier to estimate the fee for
    /// the transaction.
    ///
    /// While the gas cost can be computed exactly, the base fee can only be
    /// estimated, because the actual base fee paid by the transaction will
    /// depend on the gas prices at the time it's accepted on-chain.
    fn compute_fee_estimate(&self, gas_prices: &GasPrices, fee_tier: &FeeTier) -> Fee {
        let base_fee = gas_prices.fee(&self.gas_cost());
        base_fee.apply_tier(*fee_tier)
    }

    /// Use the provided gas prices and fee tier to refresh the fee estimate for
    /// the transaction.
    ///
    /// If the current fee estimate is too low, it will be increased. In that
    /// case, change notes will be adjusted to cover the increase if possible.
    pub fn refresh_fee_and_change<R: RngCore + CryptoRng>(
        &mut self,
        rng: R,
        gas_prices: &GasPrices,
        fee_tier: &FeeTier,
        change_address: &Address,
    ) {
        // First, refresh the change outputs, to capture any surplus imbalance.
        self.refresh_change(rng, &change_address);

        // Next, recompute the fee estimate for the actions and change outputs.
        let new_fee = self.compute_fee_estimate(gas_prices, fee_tier);

        // Update the targeted fee with the new estimate.
        if new_fee.asset_matches(&self.fee) {
            // Take the max of the current fee and the new estimate. This ensures
            // that if we already overpaid the fee for some reason, we don't lower it
            // and cause the creation of unwanted change outputs.
            self.fee.0.amount = std::cmp::max(self.fee.amount(), new_fee.amount());
        } else {
            // Otherwise, overwrite the previous fee with the new estimate.
            self.fee = new_fee;
        }

        // Finally, adjust the change outputs to cover the fee increase if possible.
        self.adjust_change_for_imbalance();
    }

    /// Return the balance of the actions in the list, without accounting for fees.
    pub fn balance_without_fee(&self) -> Balance {
        let mut balance = Balance::zero();

        for action in &self.actions {
            balance += action.balance();
        }
        for action in self.change_outputs.values() {
            balance += action.balance();
        }

        balance
    }

    /// Return the balance of the actions in the list, minus the currently estimated fee
    /// required to pay their gas costs.
    pub fn balance_with_fee(&self) -> Balance {
        self.balance_without_fee() - self.fee.0
    }

    /// Refresh the change notes used to store any surplus imbalance from the
    /// actions in the list.
    fn refresh_change<R: RngCore + CryptoRng>(&mut self, mut rng: R, change_address: &Address) {
        self.change_outputs = BTreeMap::new();
        // For each "provided" balance component, create a change note.
        for value in self.balance_with_fee().provided() {
            self.change_outputs.insert(
                value.asset_id,
                OutputPlan::new(&mut rng, value, change_address.clone()),
            );
        }
    }

    /// Attempt adjust existing change notes to repair imbalance:
    ///
    /// - cover required balance by decreasing change if possible
    /// - cover surplus balance by increasing change if possible
    fn adjust_change_for_imbalance(&mut self) {
        // We need to grab the current balance upfront before doing modifications.
        let balance = self.balance_with_fee();

        // Sweep surplus balance into existing change notes.
        for provided in balance.provided() {
            self.change_outputs
                .entry(provided.asset_id)
                .and_modify(|e| {
                    e.value.amount += provided.amount;
                });
        }

        // Attempt to cover imbalance via existing change notes.
        for required in balance.required() {
            self.change_outputs
                .entry(required.asset_id)
                .and_modify(|e| {
                    // It's important to use saturating_sub here because
                    // our expectation is that we commonly won't have enough balance.
                    e.value.amount = e.value.amount.saturating_sub(&required.amount);
                });
        }

        // Remove any 0-value change notes we might have created.
        self.change_outputs
            .retain(|_, output| output.value.amount > Amount::zero());
    }
}
