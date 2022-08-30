use std::{
    fmt::{self, Debug, Formatter},
    mem,
};

use penumbra_component::stake::{rate::RateData, validator};
use penumbra_crypto::{
    keys::AddressIndex, memo::MemoPlaintext, Address, DelegationToken, FullViewingKey, Note, Value,
    STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::view::NotesRequest;
use penumbra_tct as tct;
use penumbra_transaction::plan::{ActionPlan, OutputPlan, SpendPlan, TransactionPlan};
use penumbra_view::ViewClient;
use rand::{CryptoRng, RngCore};
use tracing::instrument;

pub use super::balance::Balance;

/// A builder for a [`TransactionPlan`] that can fill in the required spends and change outputs upon
/// finalization to make a transaction balance.
pub struct Builder<R: RngCore + CryptoRng> {
    rng: R,
    balance: Balance,
    plan: TransactionPlan,
}

impl<R: RngCore + CryptoRng> Debug for Builder<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("balance", &self.balance)
            .field("plan", &self.plan)
            .finish()
    }
}

impl<R: RngCore + CryptoRng> Builder<R> {
    /// Create a new builder.
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            balance: Balance::default(),
            plan: TransactionPlan::default(),
        }
    }

    /// Get the current transaction balance of the builder.
    pub fn balance(&self) -> &Balance {
        &self.balance
    }

    /// Set the expiry height for the transaction plan.
    #[instrument(skip(self))]
    pub fn expiry_height(&mut self, expiry_height: u64) -> &mut Self {
        self.plan.expiry_height = expiry_height;
        self
    }

    /// Add a fee to the transaction plan.
    ///
    /// Calling this function more than once will add to the fee, not replace it.
    #[instrument(skip(self))]
    pub fn fee(&mut self, fee: u64) -> &mut Self {
        self.balance.require(Value {
            amount: fee,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        });
        self.plan.fee.0 += fee;
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

    /// Add an output note from this transaction.
    ///
    /// Any unused output value will be redirected back to the originating address as change notes
    /// when the plan is [`finish`](Builder::finish)ed.
    #[instrument(skip(self, memo))]
    pub fn output(&mut self, value: Value, address: Address, memo: MemoPlaintext) -> &mut Self {
        let output = OutputPlan::new(&mut self.rng, value, address, memo).into();
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
    /// Undelegations have special rules to prevent you from accidentally locking up funds while the
    /// transaction is unbonding: any transaction containing an undelegation must contain exactly
    /// one undelegation, must spend only delegation tokens matching the validator from which the
    /// undelegation is being performed, and must output only staking tokens. This means that it
    /// must be an "exact change" transaction with no other actions.
    ///
    /// In order to ensure that the transaction is an "exact change" transaction, you should
    /// probably explicitly add the precisely correct spends to the transaction, after having
    /// generated those exact notes by splitting notes in a previous transaction, if necessary.
    ///
    /// The conditions imposed by the consensus rules are more permissive, but the builder will
    /// protect you from shooting yourself in the foot by throwing an error, should the built
    /// transaction fail these conditions.
    #[instrument(skip(self))]
    pub fn undelegate(&mut self, delegation_amount: u64, rate_data: RateData) -> &mut Self {
        let undelegation = rate_data.build_undelegate(delegation_amount).into();
        self.action(undelegation);
        self
    }

    /// Upload a validator definition in this transaction.
    #[instrument(skip(self))]
    pub fn validator_definition(&mut self, new_validator: validator::Definition) -> &mut Self {
        self.action(ActionPlan::ValidatorDefinition(new_validator.into()));
        self
    }

    fn action(&mut self, action: ActionPlan) -> &mut Self {
        use ActionPlan::*;

        // Track this action's contribution to the value balance of the transaction: this must match
        // the actual contribution to the value commitment, but this isn't checked, so make sure
        // that when you're adding a new action, you correctly match this up to the calculation of
        // the value commitment for the transaction, or else the builder will submit transactions
        // that are not balanced!
        match &action {
            Spend(spend) => self.balance.provide(spend.note.value()),
            Output(output) => self.balance.require(output.value),
            Delegate(delegate) => {
                self.balance.require(Value {
                    amount: delegate.unbonded_amount,
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                });
                self.balance.provide(Value {
                    amount: delegate.delegation_amount,
                    asset_id: DelegationToken::new(delegate.validator_identity).id(),
                })
            }
            Undelegate(undelegate) => {
                self.balance.provide(Value {
                    amount: undelegate.unbonded_amount,
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                });
                self.balance.require(Value {
                    amount: undelegate.delegation_amount,
                    asset_id: DelegationToken::new(undelegate.validator_identity).id(),
                })
            }
            ProposalSubmit(proposal_submit) => {
                self.balance.require(Value {
                    amount: proposal_submit.deposit_amount,
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                });
            }
            PositionOpen(_) => todo!(),
            PositionClose(_) => todo!(),
            PositionWithdraw(_) => todo!(),
            PositionRewardClaim(_) => todo!(),
            Swap(_) => todo!(),
            SwapClaim(_) => todo!(),
            IBCAction(_) => todo!(),
            ValidatorDefinition(_) | ProposalWithdraw(_) | DelegatorVote(_) | ValidatorVote(_) => {
                // No contribution to the value balance of the transaction
            }
        };

        // Add the action to the plan
        self.plan.actions.push(action);
        self
    }

    /// Add spends and change outputs as required to balance the transaction, using the view service
    /// provided to supply the notes and other information.
    ///
    /// Clears the contents of the builder, which can be re-used to save on allocations.
    #[instrument(skip(self, view, fvk))]
    pub async fn finish<V: ViewClient>(
        &mut self,
        view: &mut V,
        fvk: &FullViewingKey,
        source: Option<u64>,
    ) -> anyhow::Result<TransactionPlan> {
        tracing::debug!(plan = ?self.plan, balance = ?self.balance, "finalizing transaction");

        // Fill in the chain id based on the view service
        self.plan.chain_id = view.chain_params().await?.chain_id;

        let source: Option<AddressIndex> = source.map(Into::into);

        // Get all notes required to fulfill needed spends
        let mut spends = Vec::new();
        for Value { amount, asset_id } in self.balance.required() {
            spends.extend(
                view.notes(NotesRequest {
                    account_id: Some(fvk.hash().into()),
                    asset_id: Some(asset_id.into()),
                    address_index: source.map(Into::into),
                    amount_to_spend: amount,
                    include_spent: false,
                })
                .await?,
            );
        }

        // Add the required spends to the builder
        for record in spends {
            self.spend(record.note, record.position);
        }

        // For any remaining provided balance, make a single change note for each
        let self_address = fvk
            .incoming()
            .payment_address(source.unwrap_or(AddressIndex::Numeric(0)))
            .0;

        for value in self.balance.provided().collect::<Vec<_>>() {
            self.output(value, self_address, MemoPlaintext::default());
        }

        // TODO: add dummy change outputs in the staking token denomination (this means they'll pass
        // the undelegate rules check)

        // Ensure that the transaction won't cause excessive quarantining
        self.check_undelegate_rules()?;

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

        // Clear the builder and pull out the plan to return
        self.balance = Balance::new();
        let plan = mem::take(&mut self.plan);

        Ok(plan)
    }

    /// Undelegations should have a very particular form to avoid excessive quarantining: all
    /// their spends should be of the delegation token being undelegated, and all their outputs
    /// should be of the staking token, and they should contain no other actions.
    fn check_undelegate_rules(&self) -> anyhow::Result<()> {
        match self
            .plan
            .actions
            .iter()
            .filter_map(|action| {
                if let ActionPlan::Undelegate(undelegate) = action {
                    Some(undelegate)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .as_slice()
        {
            [] => {
                // No undelegations
            }
            [undelegate] => {
                let delegation_asset_id = DelegationToken::new(undelegate.validator_identity).id();
                for action in self.plan.actions.iter() {
                    match action {
                        ActionPlan::Spend(spend) => {
                            if spend.note.value().asset_id != delegation_asset_id {
                                return Err(anyhow::anyhow!(
                                    "undelegation transaction must spend only delegation tokens"
                                ));
                            }
                        }
                        ActionPlan::Output(output) => {
                            if output.value.asset_id != *STAKING_TOKEN_ASSET_ID {
                                return Err(anyhow::anyhow!(
                                    "undelegation transaction must output only staking tokens"
                                ));
                            }
                        }
                        ActionPlan::Undelegate(_) => {
                            // There's only one undelegate action, so this is the one we already
                            // know about, so we don't have to do anything with it
                        }
                        _ => {
                            return Err(anyhow::anyhow!(
                                "undelegation transaction must not contain extraneous actions"
                            ))
                        }
                    }
                }
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "undelegation transaction must not contain multiple undelegations"
                ))
            }
        }

        Ok(())
    }
}
