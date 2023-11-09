use anyhow::{anyhow, Context, Result};
use ark_ff::Zero;
use decaf377::Fr;
use decaf377_rdsa as rdsa;
use penumbra_keys::{symmetric::PayloadKey, FullViewingKey};
// use penumbra_proto::core::component::shielded_pool::v1alpha1::{SpendPlan, Spend};
use penumbra_shielded_pool::{spend, SpendPlan, OutputPlan};
use rand_core::{CryptoRng, RngCore};
use penumbra_transaction::plan::TransactionPlan; 
use penumbra_transaction::{
    action::Action,
    memo::MemoCiphertext,
    transaction::{DetectionData, TransactionParameters},
    AuthorizationData, AuthorizingData, Transaction, TransactionBody, WitnessData,
};
use penumbra_transaction::plan::ActionPlan;
// use penumbra_transaction::plan::BuildPlan;

use wasm_bindgen_test::console_log;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use crate::error::{WasmResult, WasmError, WasmOption};
use crate::utils;
use std::str::FromStr;
use penumbra_proto::core::transaction::v1alpha1 as pb;
use crate::wasm_planner::WasmPlanner;
use penumbra_proto::DomainType;
use rand_core::OsRng;
use penumbra_proto::core::transaction::v1alpha1 as ps;
use penumbra_transaction::memo;
use penumbra_proto::core::transaction::v1alpha1::MemoData;
// // #[wasm_bindgen]
#[wasm_bindgen]
pub struct WasmBuilder {}

#[wasm_bindgen]
impl WasmBuilder {

    #[wasm_bindgen]
    pub fn action_builder(
        action_plan: JsValue,
        full_viewing_key: &str,
        witness_data: JsValue,
        memo_key: JsValue, // WasmOption<JsValue
    ) -> WasmResult<JsValue> {
        console_log!("Entered action_builder!");
        
        // Serialize witness
        let witness_data_proto: pb::WitnessData = serde_wasm_bindgen::from_value(witness_data)?;
        let witness_data_conv: WitnessData = witness_data_proto.try_into()?;

        // Retrieve the viewing key
        let full_viewing_key: FullViewingKey = FullViewingKey::from_str(full_viewing_key)
            .expect("The provided string is not a valid FullViewingKey");

        // let memo_key_proto: pb::MemoData = serde_wasm_bindgen::from_value(memo_key)?;
        // let memo_key_cinv: MemoData = memo_key_proto.try_into()?;

        // let memo_key_res: PayloadKey = memo_key_cinv.encrypted_memo.try_into();

        let dummy_payload_key: PayloadKey = [0u8; 32].into();

        // Serialize the action_plan
        let action_proto: pb::ActionPlan = serde_wasm_bindgen::from_value(action_plan)?;
        let action_plan_conv: ActionPlan = action_proto.try_into()?;

        let action = match action_plan_conv {
            ActionPlan::Spend(spend_plan) => {
                console_log!("action_builder spend!");
                let spend = ActionPlan::Spend(spend_plan);
                // Some(spend.build_action(&full_viewing_key, witness_data_conv, memo_key).unwrap())
                Some(spend.build_unauth(&full_viewing_key, &witness_data_conv, Some(dummy_payload_key)).unwrap())
            }
            ActionPlan::Output(output_plan) => { 
                console_log!("action_builder output!");
                let output = ActionPlan::Output(output_plan);
                // Some(output.build_action(&full_viewing_key, witness_data_conv, memo_key).unwrap())
                Some(output.build_unauth(&full_viewing_key, &witness_data_conv, Some(dummy_payload_key)).unwrap())
            }
            _ => {
                None
            }
        };

        // Deserialize the result back to JsValue
        let action_result_proto = serde_wasm_bindgen::to_value(&Some(action))?;

        Ok(action_result_proto)
    }

    /// Build transaction in parallel
    #[wasm_bindgen]
    pub fn build_parallel(
        actions: JsValue,
        full_viewing_key: &str,
        transaction_plan: JsValue,
        witness_data: JsValue,
        auth_data: JsValue,
    ) -> WasmResult<JsValue> {
        utils::set_panic_hook();

        let plan_proto: pb::TransactionPlan = serde_wasm_bindgen::from_value(transaction_plan)?;
        let witness_data_proto: pb::WitnessData = serde_wasm_bindgen::from_value(witness_data)?;
        let witness_data_conv: WitnessData = witness_data_proto.try_into()?;
        
        let auth_data_proto: pb::AuthorizationData = serde_wasm_bindgen::from_value(auth_data)?;
        let auth_data_conv: AuthorizationData = auth_data_proto.try_into().unwrap();

        let fvk: FullViewingKey = FullViewingKey::from_str(full_viewing_key)
            .expect("The provided string is not a valid FullViewingKey");

        let plan: TransactionPlan = plan_proto.try_into()?;

        let actions_conv: Vec<Action> = serde_wasm_bindgen::from_value(actions)?;

        let transaction = plan.clone().build_unauth_with_actions(actions_conv, fvk, witness_data_conv)?;
        
        let tx = plan.authorize_with_auth(&mut OsRng, &auth_data_conv, transaction)?;

        let value = serde_wasm_bindgen::to_value(&tx.to_proto())?;

        Ok(value)
    }
}