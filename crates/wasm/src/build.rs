use crate::error::WasmResult;
use std::str::FromStr;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::utils;
use penumbra_keys::FullViewingKey;
use penumbra_transaction::{
    plan::{ActionPlan, TransactionPlan},
    WitnessData,
};

/// Builds a planned [`Action`] specified by
/// the [`ActionPlan`] in a [`TransactionPlan`].
/// Arguments:
///     transaction_plan: `TransactionPlan`
///     action_plan: `ActionPlan`
///     full_viewing_key: `bech32m String`,
///     witness_data: `WitnessData``
/// Returns: `Action`
#[wasm_bindgen]
pub fn build_action(
    transaction_plan: JsValue,
    action_plan: JsValue,
    full_viewing_key: &str,
    witness_data: JsValue,
) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let transaction_plan: TransactionPlan =
        serde_wasm_bindgen::from_value(transaction_plan.clone())?;

    let witness: WitnessData = serde_wasm_bindgen::from_value(witness_data)?;

    let action_plan: ActionPlan = serde_wasm_bindgen::from_value(action_plan)?;

    let full_viewing_key: FullViewingKey = FullViewingKey::from_str(full_viewing_key)?;

    let memo_key = transaction_plan.memo.map(|memo_plan| memo_plan.key);

    let action = ActionPlan::build_unauth(action_plan, &full_viewing_key, &witness, memo_key)?;

    let result = serde_wasm_bindgen::to_value(&action)?;
    Ok(result)
}
