use crate::error::WasmResult;
use crate::planner::Planner;
use crate::storage::IndexedDBStorage;
use crate::swap_record::SwapRecord;
use anyhow::Result;
use ark_ff::UniformRand;
use decaf377::Fq;
use penumbra_dex::swap_claim::SwapClaimPlan;
use penumbra_proto::core::chain::v1alpha1::{ChainParameters, FmdParameters};
use penumbra_proto::core::crypto::v1alpha1::{Address, DenomMetadata, Fee, StateCommitment, Value};
use penumbra_proto::core::transaction::v1alpha1::{MemoPlaintext, TransactionPlan};
use penumbra_proto::DomainType;
use rand_core::OsRng;
use serde_wasm_bindgen::Error;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub struct WasmPlanner {
    planner: Planner<OsRng>,
    storage: IndexedDBStorage,
}

#[wasm_bindgen]
impl WasmPlanner {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<WasmPlanner, Error> {
        let planner = WasmPlanner {
            planner: Planner::new(OsRng),
            storage: IndexedDBStorage::new().await?,
        };
        Ok(planner)
    }

    /// Add expiry height to plan
    /// Arguments:
    ///     expiry_height: `u64`
    #[wasm_bindgen]
    pub fn expiry_height(&mut self, expiry_height: JsValue) -> Result<(), Error> {
        self.planner
            .expiry_height(serde_wasm_bindgen::from_value(expiry_height)?);
        Ok(())
    }

    /// Add memo to plan
    /// Arguments:
    ///     memo: `MemoPlaintext`
    pub fn memo(&mut self, memo: JsValue) -> Result<(), Error> {
        self.memo_inner(memo)?;
        Ok(())
    }

    /// Add fee to plan
    /// Arguments:
    ///     fee: `Fee`
    pub fn fee(&mut self, fee: JsValue) -> Result<(), Error> {
        self.fee_inner(fee)?;
        Ok(())
    }

    /// Add output to plan
    /// Arguments:
    ///     value: `Value`
    ///     address: `Address`
    pub fn output(&mut self, value: JsValue, address: JsValue) -> Result<(), Error> {
        self.output_inner(value, address)?;
        Ok(())
    }

    /// Add swap claim to plan
    /// Arguments:
    ///     swap_commitment: `StateCommitment`
    #[wasm_bindgen]
    pub async fn swap_claim(&mut self, swap_commitment: JsValue) -> Result<(), Error> {
        self.swap_claim_inner(swap_commitment).await?;
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
    ) -> Result<(), Error> {
        self.swap_inner(input_value, into_denom, swap_claim_fee, claim_address)?;
        Ok(())
    }

    /// Build transaction plan
    /// Arguments:
    ///     self_address: `Address`
    /// Returns: `TransactionPlan`
    pub async fn plan(&mut self, self_address: JsValue) -> Result<JsValue, Error> {
        let plan = self.plan_inner(self_address).await?;
        serde_wasm_bindgen::to_value(&plan)
    }
}

impl WasmPlanner {
    fn memo_inner(&mut self, memo: JsValue) -> WasmResult<()> {
        let memo_proto: MemoPlaintext = serde_wasm_bindgen::from_value(memo)?;
        let _ = self.planner.memo(memo_proto.try_into()?);
        Ok(())
    }

    fn fee_inner(&mut self, fee: JsValue) -> WasmResult<()> {
        let fee_proto: Fee = serde_wasm_bindgen::from_value(fee)?;

        self.planner.fee(fee_proto.try_into()?);

        Ok(())
    }

    fn output_inner(&mut self, value: JsValue, address: JsValue) -> WasmResult<()> {
        let value_proto: Value = serde_wasm_bindgen::from_value(value)?;
        let address_proto: Address = serde_wasm_bindgen::from_value(address)?;

        self.planner
            .output(value_proto.try_into()?, address_proto.try_into()?);

        Ok(())
    }

    async fn swap_claim_inner(&mut self, swap_commitment: JsValue) -> WasmResult<()> {
        let swap_commitment_proto: StateCommitment =
            serde_wasm_bindgen::from_value(swap_commitment)?;

        let swap_record: SwapRecord = self
            .storage
            .get_swap_by_commitment(swap_commitment_proto)
            .await?
            .expect("Swap record not found")
            .try_into()?;
        let chain_params_proto: ChainParameters = self
            .storage
            .get_chain_parameters()
            .await?
            .expect("Chain params not found");

        let swap_claim_plan = SwapClaimPlan {
            swap_plaintext: swap_record.swap,
            position: swap_record.position,
            output_data: swap_record.output_data,
            epoch_duration: chain_params_proto.epoch_duration,
            proof_blinding_r: Fq::rand(&mut OsRng),
            proof_blinding_s: Fq::rand(&mut OsRng),
        };

        self.planner.swap_claim(swap_claim_plan);
        Ok(())
    }

    fn swap_inner(
        &mut self,
        input_value: JsValue,
        into_denom: JsValue,
        swap_claim_fee: JsValue,
        claim_address: JsValue,
    ) -> WasmResult<()> {
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

    async fn plan_inner(&mut self, self_address: JsValue) -> WasmResult<TransactionPlan> {
        let self_address_proto: Address = serde_wasm_bindgen::from_value(self_address)?;

        let chain_params_proto: ChainParameters = self
            .storage
            .get_chain_parameters()
            .await?
            .expect("No found chain params");
        let fmd_params_proto: FmdParameters = self
            .storage
            .get_fmd_parameters()
            .await?
            .expect("No found fmd");

        let mut spendable_notes = Vec::new();

        let (spendable_requests, _) = self.planner.notes_requests();

        let idb_storage = IndexedDBStorage::new().await?;
        for request in spendable_requests {
            let notes = idb_storage.get_notes(request);
            spendable_notes.extend(notes.await?);
        }

        // Plan the transaction using the gathered information

        let plan: penumbra_transaction::plan::TransactionPlan =
            self.planner.plan_with_spendable_and_votable_notes(
                &chain_params_proto.try_into()?,
                &fmd_params_proto.try_into()?,
                spendable_notes,
                Vec::new(),
                self_address_proto.try_into()?,
            )?;

        let plan_proto: TransactionPlan = plan.to_proto();

        Ok(plan_proto)
    }
}
