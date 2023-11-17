use crate::error::WasmResult;
use crate::planner::Planner;
use crate::storage::IndexedDBStorage;
use crate::swap_record::SwapRecord;
use crate::utils;
use anyhow::{anyhow, Context};
use ark_ff::UniformRand;
use decaf377::Fq;
use penumbra_chain::params::{ChainParameters, FmdParameters};
use penumbra_dex::{swap_claim::SwapClaimPlan, lp::position};
use penumbra_governance::{proposal, delegator_vote};
use penumbra_keys::{symmetric::PayloadKey, FullViewingKey};
use penumbra_proto::{
    core::{
        asset::v1alpha1::{DenomMetadata, Value},
        component::fee::v1alpha1::{Fee, GasPrices},
        component::ibc::v1alpha1::Ics20Withdrawal,
        keys::v1alpha1::Address,
        transaction::v1alpha1 as pb,
        transaction::v1alpha1::MemoPlaintext,
        transaction::v1alpha1::TransactionPlan as tp,
    },
    crypto::tct::v1alpha1::StateCommitment,
    DomainType,
};
use penumbra_transaction::{plan::ActionPlan, plan::TransactionPlan, WitnessData};
use rand_core::OsRng;
use wasm_bindgen_test::console_log;
use std::str::FromStr;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use penumbra_transaction::action::Action;

#[wasm_bindgen]
pub struct WasmPlanner {
    planner: Planner<OsRng>,
    storage: IndexedDBStorage,
    chain_params: ChainParameters,
    fmd_params: FmdParameters,
}

#[wasm_bindgen]
impl WasmPlanner {
    /// Create new instances of `WasmPlanner`
    /// Function opens a connection to indexedDb
    /// Arguments:
    ///     idb_constants: `IndexedDbConstants`
    ///     chain_params: `ChainParams`
    ///     fmd_params: `FmdParameters`
    /// Returns: `WasmPlanner`
    #[wasm_bindgen]
    pub async fn new(
        idb_constants: JsValue,
        chain_params: JsValue,
        fmd_params: JsValue,
    ) -> WasmResult<WasmPlanner> {
        utils::set_panic_hook();

        let constants = serde_wasm_bindgen::from_value(idb_constants)?;
        let planner = WasmPlanner {
            planner: Planner::new(OsRng),
            storage: IndexedDBStorage::new(constants).await?,
            chain_params: serde_wasm_bindgen::from_value(chain_params)?,
            fmd_params: serde_wasm_bindgen::from_value(fmd_params)?,
        };
        Ok(planner)
    }

    /// Builds a planned [`Action`] specified by
    /// the [`ActionPlan`] in a [`TransactionPlan`].
    /// Arguments:
    ///     &self: `WasmPlanner`
    ///     transaction_plan: `TransactionPlan`
    ///     action_plan: `ActionPlan`
    ///     full_viewing_key: `bech32m String`,
    ///     witness_data: `WitnessData``
    /// Returns: `Action`
    #[wasm_bindgen]
    pub fn build_action(
        &self,
        transaction_plan: JsValue,
        action_plan: JsValue,
        full_viewing_key: &str,
        witness_data: JsValue,
    ) -> WasmResult<JsValue> {
        utils::set_panic_hook();

        let transaction_plan_proto: tp = serde_wasm_bindgen::from_value(transaction_plan.clone())?;
        let transaction_plan_: TransactionPlan = transaction_plan_proto.try_into()?;

        let witness_data_proto: pb::WitnessData = serde_wasm_bindgen::from_value(witness_data)?;
        let witness_data_: WitnessData = witness_data_proto.try_into()?;

        let action_proto: pb::ActionPlan = serde_wasm_bindgen::from_value(action_plan)?;
        let action_plan_: ActionPlan = action_proto.try_into()?;

        let full_viewing_key: FullViewingKey = FullViewingKey::from_str(full_viewing_key)
            .expect("The provided string is not a valid FullViewingKey");

        let mut memo_key: Option<PayloadKey> = None;
        if transaction_plan_.memo_plan.is_some() {
            let memo_plan = transaction_plan_
                .memo_plan
                .clone()
                .ok_or_else(|| anyhow!("missing memo_plan in TransactionPlan"))?;
            memo_key = Some(memo_plan.key);
        }

        // 
        let action = match action_plan_ {
            ActionPlan::Spend(spend_plan) => {
                let spend = ActionPlan::Spend(spend_plan);
                Some(
                    spend
                        .build_unauth(&full_viewing_key, &witness_data_, memo_key)
                        .expect("Build spend action failed!"),
                )
            }
            ActionPlan::Output(output_plan) => {
                let output = ActionPlan::Output(output_plan);
                Some(
                    output
                        .build_unauth(&full_viewing_key, &witness_data_, memo_key)
                        .expect("Build output action failed!"),
                )
            }

            // TODO: Other action variants besides 'Spend' and 'Output' still require testing. 
            ActionPlan::Swap(swap_plan) => {
                let swap = ActionPlan::Swap(swap_plan);
                Some(
                    swap
                        .build_unauth(&full_viewing_key, &witness_data_, memo_key)
                        .expect("Build output action failed!"),
                )
            }
            ActionPlan::SwapClaim(swap_claim_plan) => {
                let swap_claim = ActionPlan::SwapClaim(swap_claim_plan);
                Some(
                    swap_claim
                        .build_unauth(&full_viewing_key, &witness_data_, memo_key)
                        .expect("Build output action failed!"),
                )
            }
            ActionPlan::Delegate(delegation) => {
                Some(Action::Delegate(delegation))
            }
            ActionPlan::Undelegate(undelegation) => {
                Some(Action::Undelegate(undelegation))
            }
            ActionPlan::UndelegateClaim(undelegate_claim) => {
                let undelegate_claim = undelegate_claim.undelegate_claim();
                Some(Action::UndelegateClaim(undelegate_claim))
            }
            ActionPlan::ProposalSubmit(proposal_submit) => {
                Some(Action::ProposalSubmit(proposal_submit))
            }
            ActionPlan::ProposalWithdraw(proposal_withdraw) => {
                Some(Action::ProposalWithdraw(proposal_withdraw))
            }
            ActionPlan::ValidatorVote(validator_vote) => {
                Some(Action::ValidatorVote(validator_vote))
            }
            ActionPlan::DelegatorVote(delegator_vote) => {
                let note_commitment = delegator_vote.staked_note.commit();
                let auth_path = witness_data_
                    .state_commitment_proofs
                    .get(&note_commitment)
                    .context(format!("could not get proof for {note_commitment:?}"))?;

                Some(Action::DelegatorVote(delegator_vote.delegator_vote(
                    &full_viewing_key,
                    [0; 64].into(),
                    auth_path.clone(),
                )))
            }
            ActionPlan::ProposalDepositClaim(proposal_deposit_claim) => {
                Some(Action::ProposalDepositClaim(proposal_deposit_claim))
            }
            ActionPlan::ValidatorDefinition(validator_definition) => {
                Some(Action::ValidatorDefinition(validator_definition))
            }
            ActionPlan::IbcAction(ibc_action) => {
                Some(Action::IbcRelay(ibc_action))
            }
            ActionPlan::DaoSpend(dao_spend) => {
                Some(Action::DaoSpend(dao_spend))
            }
            ActionPlan::DaoOutput(dao_output) => {
                Some(Action::DaoOutput(dao_output))
            }
            ActionPlan::DaoDeposit(dao_deposit) => {
                Some(Action::DaoDeposit(dao_deposit))
            }
            ActionPlan::PositionOpen(position_open) => {
                Some(Action::PositionOpen(position_open))
            }
            ActionPlan::PositionClose(position_close) => {
                Some(Action::PositionClose(position_close))
            }
            ActionPlan::PositionWithdraw(position_withdrawn) => {
                Some(Action::PositionWithdraw(position_withdrawn.position_withdraw()))
            }
            ActionPlan::Withdrawal(ics20_withdrawal) => {
                Some(Action::Ics20Withdrawal(ics20_withdrawal))
            }
            // TODO: Should we handle `PositionRewardClaim`?
            _ => None,
        };

        let action_result_proto = serde_wasm_bindgen::to_value(&Some(action))?;
        Ok(action_result_proto)
    }

    /// Public getter for the 'storage' field
    pub fn get_storage(&self) -> *const IndexedDBStorage {
        &self.storage
    }

    /// Add expiry height to plan
    /// Arguments:
    ///     expiry_height: `u64`
    #[wasm_bindgen]
    pub fn expiry_height(&mut self, expiry_height: u64) -> WasmResult<()> {
        utils::set_panic_hook();

        self.planner.expiry_height(expiry_height);
        Ok(())
    }

    /// Set gas prices
    /// Arguments:
    ///     gas_prices: `GasPrices`
    pub fn set_gas_prices(&mut self, gas_prices: JsValue) -> WasmResult<()> {
        let gas_prices_proto: GasPrices = serde_wasm_bindgen::from_value(gas_prices)?;
        self.planner.set_gas_prices(gas_prices_proto.try_into()?);
        Ok(())
    }

    /// Add memo to plan
    /// Arguments:
    ///     memo: `MemoPlaintext`
    pub fn memo(&mut self, memo: JsValue) -> WasmResult<()> {
        utils::set_panic_hook();
        let memo_proto: MemoPlaintext = serde_wasm_bindgen::from_value(memo)?;
        let _ = self.planner.memo(memo_proto.try_into()?);
        Ok(())
    }

    /// Add fee to plan
    /// Arguments:
    ///     fee: `Fee`
    pub fn fee(&mut self, fee: JsValue) -> WasmResult<()> {
        utils::set_panic_hook();

        let fee_proto: Fee = serde_wasm_bindgen::from_value(fee)?;
        self.planner.fee(fee_proto.try_into()?);

        Ok(())
    }

    /// Add output to plan
    /// Arguments:
    ///     value: `Value`
    ///     address: `Address`
    pub fn output(&mut self, value: JsValue, address: JsValue) -> WasmResult<()> {
        utils::set_panic_hook();

        let value_proto: Value = serde_wasm_bindgen::from_value(value)?;
        let address_proto: Address = serde_wasm_bindgen::from_value(address)?;

        self.planner
            .output(value_proto.try_into()?, address_proto.try_into()?);

        Ok(())
    }

    /// Add swap claim to plan
    /// Arguments:
    ///     swap_commitment: `StateCommitment`
    #[wasm_bindgen]
    pub async fn swap_claim(&mut self, swap_commitment: JsValue) -> WasmResult<()> {
        utils::set_panic_hook();

        let swap_commitment_proto: StateCommitment =
            serde_wasm_bindgen::from_value(swap_commitment)?;

        let swap_record: SwapRecord = self
            .storage
            .get_swap_by_commitment(swap_commitment_proto)
            .await?
            .expect("Swap record not found")
            .try_into()?;

        let swap_claim_plan = SwapClaimPlan {
            swap_plaintext: swap_record.swap,
            position: swap_record.position,
            output_data: swap_record.output_data,
            epoch_duration: self.chain_params.epoch_duration,
            proof_blinding_r: Fq::rand(&mut OsRng),
            proof_blinding_s: Fq::rand(&mut OsRng),
        };

        self.planner.swap_claim(swap_claim_plan);
        Ok(())
    }

    /// Add swap  to plan
    /// Arguments:
    ///     input_value: `Value`
    ///     into_denom: `DenomMetadata`
    ///     swap_claim_fee: `Fee`
    ///     claim_address: `Address`
    pub fn swap(
        &mut self,
        input_value: JsValue,
        into_denom: JsValue,
        swap_claim_fee: JsValue,
        claim_address: JsValue,
    ) -> WasmResult<()> {
        utils::set_panic_hook();

        let input_value_proto: Value = serde_wasm_bindgen::from_value(input_value)?;
        let into_denom_proto: DenomMetadata = serde_wasm_bindgen::from_value(into_denom)?;
        let swap_claim_fee_proto: Fee = serde_wasm_bindgen::from_value(swap_claim_fee)?;
        let claim_address_proto: Address = serde_wasm_bindgen::from_value(claim_address)?;

        let _ = self.planner.swap(
            input_value_proto.try_into()?,
            into_denom_proto.try_into()?,
            swap_claim_fee_proto.try_into()?,
            claim_address_proto.try_into()?,
        );

        Ok(())
    }

    /// Add ICS20 withdrawal to plan
    /// Arguments:
    ///     withdrawal: `Ics20Withdrawal`
    pub fn ics20_withdrawal(&mut self, withdrawal: JsValue) -> WasmResult<()> {
        let withdrawal_proto: Ics20Withdrawal = serde_wasm_bindgen::from_value(withdrawal)?;
        self.planner.ics20_withdrawal(withdrawal_proto.try_into()?);
        Ok(())
    }

    /// Builds transaction plan.
    /// Refund address provided in the case there is extra balances to be returned.
    /// Arguments:
    ///     refund_address: `Address`
    /// Returns: `TransactionPlan`
    pub async fn plan(&mut self, refund_address: JsValue) -> WasmResult<JsValue> {
        utils::set_panic_hook();

        // Calculate the gas that needs to be paid for the transaction based on the configured gas prices.
        // Note that _paying the fee might incur an additional `Spend` action_, thus increasing the fee,
        // so we slightly overpay here and then capture the excess as change later during `plan_with_spendable_and_votable_notes`.
        // Add the fee to the planner's internal balance.
        self.planner.add_gas_fees();

        let mut spendable_notes = Vec::new();

        let (spendable_requests, _) = self.planner.notes_requests();
        for request in spendable_requests {
            let notes = self.storage.get_notes(request);
            spendable_notes.extend(notes.await?);
        }

        // Plan the transaction using the gathered information
        let refund_address_proto: Address = serde_wasm_bindgen::from_value(refund_address)?;
        let plan = self.planner.plan_with_spendable_and_votable_notes(
            &self.chain_params,
            &self.fmd_params,
            spendable_notes,
            Vec::new(),
            refund_address_proto.try_into()?,
        )?;

        let plan_proto = plan.to_proto();
        Ok(serde_wasm_bindgen::to_value(&plan_proto)?)
    }
}
