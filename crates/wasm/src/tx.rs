use std::convert::TryInto;
use std::str::FromStr;

use penumbra_chain::params::{ChainParameters, FmdParameters};
use penumbra_keys::{Address, FullViewingKey};

use penumbra_keys::keys::{AddressIndex, SpendKey};
use penumbra_tct::{Proof, StateCommitment, Tree};
use penumbra_transaction::plan::TransactionPlan;
use penumbra_transaction::{AuthorizationData, Transaction, WitnessData};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::note_record::SpendableNoteRecord;
use crate::planner::Planner;
use crate::utils;
use crate::view_server::{load_tree, StoredTree};
use web_sys::console as web_console;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendTx {
    notes: Vec<SpendableNoteRecord>,
    chain_parameters: penumbra_proto::core::chain::v1alpha1::ChainParameters,
    fmd_parameters: penumbra_proto::core::chain::v1alpha1::FmdParameters,
}

#[wasm_bindgen]
pub fn send_plan(
    full_viewing_key: &str,
    value_js: JsValue,
    dest_address: &str,
    view_service_data: JsValue,
) -> JsValue {
    utils::set_panic_hook();
    web_console::log_1(&value_js);

    let value: penumbra_proto::core::crypto::v1alpha1::Value =
        serde_wasm_bindgen::from_value(value_js).expect("able to parse send plan's Value from JS");

    let address =
        Address::from_str(dest_address).expect("send plan's destination address is valid");
    let mut planner = Planner::new(OsRng);
    planner.fee(Default::default());
    planner.output(
        value.try_into().expect("encoded protobuf Value is valid"),
        address,
    );

    let fvk = FullViewingKey::from_str(full_viewing_key)
        .expect("the provided string is a valid FullViewingKey");

    let send_tx: SendTx = serde_wasm_bindgen::from_value(view_service_data)
        .expect("able to parse send plan's SendTx from JS");

    let chain_params: ChainParameters = send_tx
        .chain_parameters
        .try_into()
        .expect("encoded protobuf ChainParameters is valid");
    let fmd_params: FmdParameters = send_tx
        .fmd_parameters
        .try_into()
        .expect("encoded protobuf FmdParameters is valid");

    let plan = planner
        .plan_with_spendable_and_votable_notes(
            &chain_params,
            &fmd_params,
            &fvk,
            AddressIndex::from(0u32),
            send_tx.notes,
            Default::default(),
        )
        .expect("valid send transaction parameters were provided");

    serde_wasm_bindgen::to_value(&plan).expect("able to serialize send plan to JS")
}

#[wasm_bindgen]
pub fn encode_tx(transaction: JsValue) -> JsValue {
    utils::set_panic_hook();
    let tx: Transaction = serde_wasm_bindgen::from_value(transaction)
        .expect("able to deserialize transaction from JS");
    let tx_encoding: Vec<u8> = tx.try_into().expect("able to encode transaction to bytes");
    serde_wasm_bindgen::to_value(&tx_encoding)
        .expect("able to serialize transaction encoding to JS")
}

#[wasm_bindgen]
pub fn build_tx(
    spend_key_str: &str,
    full_viewing_key: &str,
    transaction_plan: JsValue,
    stored_tree: JsValue,
) -> JsValue {
    utils::set_panic_hook();
    let plan: TransactionPlan = serde_wasm_bindgen::from_value(transaction_plan)
        .expect("able to deserialize transaction plan from JS");

    let fvk = FullViewingKey::from_str(full_viewing_key)
        .expect("the provided string is a valid FullViewingKey");

    let auth_data = sign_plan(spend_key_str, plan.clone());

    let stored_tree: StoredTree = serde_wasm_bindgen::from_value(stored_tree)
        .expect("able to deserialize stored tree from JS");

    let nct = load_tree(stored_tree);

    let witness_data = witness(nct, plan.clone());

    let tx = build_transaction(&fvk, plan.clone(), auth_data, witness_data);

    serde_wasm_bindgen::to_value(&tx).expect("able to serialize transaction to JS")
}

pub fn sign_plan(spend_key_str: &str, transaction_plan: TransactionPlan) -> AuthorizationData {
    let spend_key = SpendKey::from_str(spend_key_str).expect("spend key is valid");

    transaction_plan.authorize(OsRng, &spend_key)
}

pub fn build_transaction(
    fvk: &FullViewingKey,
    plan: TransactionPlan,
    auth_data: AuthorizationData,
    witness_data: WitnessData,
) -> Transaction {
    plan.build(fvk, witness_data)
        .expect("valid transaction plan was provided")
        .authorize(&mut OsRng, &auth_data)
        .expect("valid authorization data was provided")
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
        .map(|nc| nct.witness(*nc).expect("note commitment is in the NCT"))
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
    witness_data
}
