use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryInto;
use std::str::FromStr;

use anyhow::anyhow;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Error;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use penumbra_keys::keys::SpendKey;
use penumbra_keys::FullViewingKey;
use penumbra_proto::core::transaction::v1 as pb;
use penumbra_proto::core::transaction::v1::{TransactionPerspective, TransactionView};
use penumbra_proto::DomainType;
use penumbra_tct::{Proof, StateCommitment};
use penumbra_transaction::plan::TransactionPlan;
use penumbra_transaction::Action;
use penumbra_transaction::{AuthorizationData, Transaction, WitnessData};

use penumbra_extension::error::WasmResult;
use penumbra_extension::storage::IndexedDBStorage;
use penumbra_extension::storage::IndexedDbConstants;
use penumbra_extension::tx::transaction_info_inner;
use penumbra_extension::utils;
use penumbra_extension::view_server::{load_tree, StoredTree};

/// encode transaction to bytes
/// Arguments:
///     transaction: `penumbra_transaction::Transaction`
/// Returns: `<Vec<u8>`
#[wasm_bindgen]
pub fn encode_tx(transaction: JsValue) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let tx: Transaction = serde_wasm_bindgen::from_value(transaction)?;
    let tx_encoding: Vec<u8> = tx.try_into()?;
    let result = serde_wasm_bindgen::to_value(&tx_encoding)?;
    Ok(result)
}

/// decode base64 bytes to transaction
/// Arguments:
///     tx_bytes: `base64 String`
/// Returns: `penumbra_transaction::Transaction`
#[wasm_bindgen]
pub fn decode_tx(tx_bytes: &str) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let tx_vec: Vec<u8> =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, tx_bytes)?;
    let transaction: Transaction = Transaction::try_from(tx_vec)?;
    let result = serde_wasm_bindgen::to_value(&transaction)?;
    Ok(result)
}

/// authorize transaction (sign  transaction using  spend key)
/// Arguments:
///     spend_key_str: `bech32m String`
///     transaction_plan: `pb::TransactionPlan`
/// Returns: `pb::AuthorizationData`
#[wasm_bindgen]
pub fn authorize(spend_key_str: &str, transaction_plan: JsValue) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let plan_proto: pb::TransactionPlan = serde_wasm_bindgen::from_value(transaction_plan)?;
    let spend_key = SpendKey::from_str(spend_key_str)?;
    let plan: TransactionPlan = plan_proto.try_into()?;

    let auth_data: AuthorizationData = plan.authorize(OsRng, &spend_key)?;
    let result = serde_wasm_bindgen::to_value(&auth_data.to_proto())?;
    Ok(result)
}

/// Get witness data
/// Obtaining witness data is directly related to SCT so we need to pass the tree data
/// Arguments:
///     transaction_plan: `pb::TransactionPlan`
///     stored_tree: `StoredTree`
/// Returns: `pb::WitnessData`
#[wasm_bindgen]
pub fn witness(transaction_plan: JsValue, stored_tree: JsValue) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let plan_proto: pb::TransactionPlan = serde_wasm_bindgen::from_value(transaction_plan)?;

    let plan: TransactionPlan = plan_proto.try_into()?;

    let stored_tree: StoredTree = serde_wasm_bindgen::from_value(stored_tree)?;

    let sct = load_tree(stored_tree);

    let note_commitments: Vec<StateCommitment> = plan
        .spend_plans()
        .filter(|plan| plan.note.amount() != 0u64.into())
        .map(|spend| spend.note.commit())
        .chain(
            plan.swap_claim_plans()
                .map(|swap_claim| swap_claim.swap_plaintext.swap_commitment()),
        )
        .collect();

    let anchor = sct.root();

    // Obtain an auth path for each requested note commitment

    let auth_paths = note_commitments
        .iter()
        .map(|nc| {
            sct.witness(*nc)
                .ok_or(anyhow!("note commitment is in the SCT"))
        })
        .collect::<Result<Vec<Proof>, anyhow::Error>>()?;

    // Release the read lock on the SCT
    drop(sct);

    let mut witness_data = WitnessData {
        anchor,
        state_commitment_proofs: auth_paths
            .into_iter()
            .map(|proof| (proof.commitment(), proof))
            .collect(),
    };

    // Now we need to augment the witness data with dummy proofs such that
    // note commitments corresponding to dummy spends also have proofs.
    for nc in plan
        .spend_plans()
        .filter(|plan| plan.note.amount() == 0u64.into())
        .map(|plan| plan.note.commit())
    {
        witness_data.add_proof(nc, Proof::dummy(&mut OsRng, nc));
    }

    let result = serde_wasm_bindgen::to_value(&witness_data.to_proto())?;
    Ok(result)
}

/// Build serial tx
/// Building a transaction may take some time,
/// depending on CPU performance and number of actions in transaction_plan
/// Arguments:
///     full_viewing_key: `bech32m String`
///     transaction_plan: `pb::TransactionPlan`
///     witness_data: `pb::WitnessData`
///     auth_data: `pb::AuthorizationData`
/// Returns: `pb::Transaction`
#[wasm_bindgen]
pub fn build(
    full_viewing_key: &str,
    transaction_plan: JsValue,
    witness_data: JsValue,
    auth_data: JsValue,
) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let plan_proto: pb::TransactionPlan = serde_wasm_bindgen::from_value(transaction_plan)?;
    let witness_data_proto: pb::WitnessData = serde_wasm_bindgen::from_value(witness_data)?;
    let auth_data_proto: pb::AuthorizationData = serde_wasm_bindgen::from_value(auth_data)?;

    let fvk = FullViewingKey::from_str(full_viewing_key)?;

    let plan: TransactionPlan = plan_proto.try_into()?;

    let tx: Transaction = plan.build(
        &fvk,
        &witness_data_proto.try_into()?,
        &auth_data_proto.try_into()?,
    )?;

    let value = serde_wasm_bindgen::to_value(&tx.to_proto())?;

    Ok(value)
}

/// Build parallel tx
/// Building a transaction may take some time,
/// depending on CPU performance and number of actions in transaction_plan
/// Arguments:
///     actions: `Vec<Actions>`
///     transaction_plan: `pb::TransactionPlan`
///     witness_data: `pb::WitnessData`
///     auth_data: `pb::AuthorizationData`
/// Returns: `pb::Transaction`
#[wasm_bindgen]
pub fn build_parallel(
    actions: JsValue,
    transaction_plan: JsValue,
    witness_data: JsValue,
    auth_data: JsValue,
) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let plan: TransactionPlan = serde_wasm_bindgen::from_value(transaction_plan.clone())?;

    let witness_data_proto: pb::WitnessData = serde_wasm_bindgen::from_value(witness_data)?;
    let witness_data: WitnessData = witness_data_proto.try_into()?;

    let auth_data_proto: pb::AuthorizationData = serde_wasm_bindgen::from_value(auth_data)?;
    let auth_data: AuthorizationData = auth_data_proto.try_into()?;

    let actions: Vec<Action> = serde_wasm_bindgen::from_value(actions)?;

    let transaction = plan
        .clone()
        .build_unauth_with_actions(actions, &witness_data)?;

    let tx = plan.apply_auth_data(&mut OsRng, &auth_data, transaction)?;

    let value = serde_wasm_bindgen::to_value(&tx.to_proto())?;

    Ok(value)
}

/// Get transaction view, transaction perspective
/// Arguments:
///     full_viewing_key: `bech32 String`
///     tx: `pbt::Transaction`
///     idb_constants: IndexedDbConstants
/// Returns: `TxInfoResponse`
#[wasm_bindgen]
pub async fn transaction_info(
    full_viewing_key: &str,
    tx: JsValue,
    idb_constants: JsValue,
) -> Result<JsValue, Error> {
    utils::set_panic_hook();

    let transaction = serde_wasm_bindgen::from_value(tx)?;
    let constants = serde_wasm_bindgen::from_value(idb_constants)?;
    let response = transaction_info_inner(full_viewing_key, transaction, constants).await?;

    serde_wasm_bindgen::to_value(&response)
}
