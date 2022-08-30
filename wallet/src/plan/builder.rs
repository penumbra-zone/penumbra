use std::mem;

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

use super::balance::Balance;

pub struct Builder<R: RngCore + CryptoRng> {
    rng: R,
    balance: Balance,
    plan: TransactionPlan,
}

impl<R: RngCore + CryptoRng> Builder<R> {
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            balance: Balance::default(),
            plan: TransactionPlan::default(),
        }
    }

    pub fn expiry_height(&mut self, expiry_height: u64) -> &mut Self {
        self.plan.expiry_height = expiry_height;
        self
    }

    pub fn fee(&mut self, fee: u64) -> &mut Self {
        self.balance.require(Value {
            amount: fee,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        });
        self.plan.fee.0 += fee;
        self
    }

    pub fn spend(&mut self, note: Note, position: tct::Position) -> &mut Self {
        let spend = SpendPlan::new(&mut self.rng, note, position).into();
        self.action(spend);
        self
    }

    pub fn output(&mut self, value: Value, address: Address, memo: MemoPlaintext) -> &mut Self {
        let output = OutputPlan::new(&mut self.rng, value, address, memo).into();
        self.action(output);
        self
    }

    pub fn delegate(&mut self, unbonded_amount: u64, rate_data: RateData) -> &mut Self {
        let delegation = rate_data.build_delegate(unbonded_amount).into();
        self.action(delegation);
        self
    }

    pub fn undelegate(&mut self, delegation_amount: u64, rate_data: RateData) -> &mut Self {
        let undelegation = rate_data.build_undelegate(delegation_amount).into();
        self.action(undelegation);
        self
    }

    pub fn validator_definition(&mut self, new_validator: validator::Definition) -> &mut Self {
        self.action(ActionPlan::ValidatorDefinition(new_validator.into()));
        self
    }

    fn action(&mut self, action: ActionPlan) -> &mut Self {
        use ActionPlan::*;

        // Track this action's contribution to the value balance of the transaction
        match &action {
            Spend(spend) => self.balance.require(spend.note.value()),
            Output(output) => self.balance.provide(output.value),
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

    pub async fn finish<V: ViewClient>(
        &mut self,
        view: &mut V,
        fvk: &FullViewingKey,
        source: Option<u64>,
    ) -> anyhow::Result<TransactionPlan> {
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

        // TODO: dummy change outputs

        // Now the transaction should be fully balanced
        assert!(self.balance.is_zero());

        // Add clue plans for `Output`s.
        let fmd_params = view.fmd_parameters().await?;
        let precision_bits = fmd_params.precision_bits;
        self.plan
            .add_all_clue_plans(&mut self.rng, precision_bits.into());

        Ok(mem::take(&mut self.plan))
    }
}
