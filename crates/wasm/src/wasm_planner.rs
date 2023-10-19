use ark_ff::UniformRand;
use decaf377::Fq;
use rand_core::OsRng;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use penumbra_chain::params::{ChainParameters, FmdParameters};
use penumbra_dex::swap_claim::SwapClaimPlan;
use penumbra_proto::core::asset::v1alpha1::{DenomMetadata, Value};
use penumbra_proto::core::component::fee::v1alpha1::{Fee, GasPrices};
use penumbra_proto::core::component::ibc::v1alpha1::Ics20Withdrawal;
use penumbra_proto::core::keys::v1alpha1::Address;
use penumbra_proto::core::transaction::v1alpha1::MemoPlaintext;
use penumbra_proto::crypto::tct::v1alpha1::StateCommitment;
use penumbra_proto::DomainType;

use crate::error::WasmResult;
use crate::planner::Planner;
use crate::storage::IndexedDBStorage;
use crate::swap_record::SwapRecord;
use crate::utils;

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
