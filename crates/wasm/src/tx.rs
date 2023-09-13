use anyhow::Context;
use std::convert::TryInto;
use std::str::FromStr;

use penumbra_keys::keys::SpendKey;
use penumbra_keys::FullViewingKey;
use penumbra_tct::{Proof, StateCommitment, Tree};
use penumbra_transaction::plan::TransactionPlan;
use penumbra_transaction::{AuthorizationData, Transaction, WitnessData};
use rand_core::OsRng;

use crate::error::WasmResult;
use crate::utils;
use crate::view_server::{load_tree, StoredTree};
use serde_wasm_bindgen::Error;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

/// encode transaction to bytes
/// Arguments:
///     transaction: `penumbra_transaction::Transaction`
/// Returns: `<Vec<u8>`
#[wasm_bindgen(js_name = encodeTx)]
pub fn encode_tx(transaction: JsValue) -> Result<JsValue, Error> {
    let result = encode_transaction(transaction)?;
    serde_wasm_bindgen::to_value(&result)
}

/// decode base64 bytes to transaction
/// Arguments:
///     tx_bytes: `base64 String`
/// Returns: `penumbra_transaction::Transaction`
#[wasm_bindgen(js_name = decodeTx)]
pub fn decode_tx(tx_bytes: &str) -> Result<JsValue, Error> {
    let result = decode_transaction(tx_bytes)?;
    serde_wasm_bindgen::to_value(&result)
}

/// deprecated
/// In the future, this function will be split into separate functions
/// - sign the transaction
/// - build transaction
/// - get wittness
#[wasm_bindgen]
pub fn build_tx(
    spend_key_str: &str,
    full_viewing_key: &str,
    transaction_plan: JsValue,
    stored_tree: JsValue,
) -> JsValue {
    utils::set_panic_hook();
    let plan: TransactionPlan = serde_wasm_bindgen::from_value(transaction_plan).unwrap();

    let fvk = FullViewingKey::from_str(full_viewing_key.as_ref())
        .context("The provided string is not a valid FullViewingKey")
        .unwrap();

    let auth_data = sign_plan(spend_key_str, plan.clone());

    let stored_tree: StoredTree = serde_wasm_bindgen::from_value(stored_tree).unwrap();

    let nct = load_tree(stored_tree);

    let witness_data = witness(nct, plan.clone());

    let tx = build_transaction(&fvk, plan.clone(), auth_data, witness_data);

    return serde_wasm_bindgen::to_value(&tx).unwrap();
}

pub fn sign_plan(spend_key_str: &str, transaction_plan: TransactionPlan) -> AuthorizationData {
    let spend_key = SpendKey::from_str(spend_key_str).unwrap();

    let authorization_data = transaction_plan.authorize(OsRng, &spend_key);
    return authorization_data;
}

pub fn build_transaction(
    fvk: &FullViewingKey,
    plan: TransactionPlan,
    auth_data: AuthorizationData,
    witness_data: WitnessData,
) -> Transaction {
    return plan
        .build(fvk, witness_data)
        .unwrap()
        .authorize(&mut OsRng, &auth_data)
        .unwrap();
}

fn witness(nct: Tree, plan: TransactionPlan) -> WitnessData {
    let note_commitments: Vec<StateCommitment> = plan
        .spend_plans()
        .filter(|plan| plan.note.amount() != 0u64.into())
        .map(|spend| spend.note.commit().into())
        .chain(
            plan.swap_claim_plans()
                .map(|swap_claim| swap_claim.swap_plaintext.swap_commitment().into()),
        )
        .collect();

    let anchor = nct.root();

    // Obtain an auth path for each requested note commitment

    let auth_paths: Vec<Proof> = note_commitments
        .iter()
        .map(|nc| nct.witness(*nc).unwrap())
        .collect::<Vec<Proof>>();

    // Release the read lock on the NCT
    drop(nct);

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
    return witness_data;
}

pub fn encode_transaction(transaction: JsValue) -> WasmResult<Vec<u8>> {
    let tx: Transaction = serde_wasm_bindgen::from_value(transaction)?;
    let tx_encoding: Vec<u8> = tx.try_into()?;
    Ok(tx_encoding)
}

pub fn decode_transaction(tx_bytes: &str) -> WasmResult<Transaction> {
    let tx_vec: Vec<u8> =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, tx_bytes)?;
    let transaction: Transaction = Transaction::try_from(tx_vec)?;
    Ok(transaction)
}
