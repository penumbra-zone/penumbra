use crate::error::WasmResult;
use crate::storage::IndexedDBStorage;
use crate::view_server::{load_tree, StoredTree};
use penumbra_keys::keys::SpendKey;
use penumbra_keys::FullViewingKey;
use penumbra_proto::core::transaction::v1alpha1::{TransactionPerspective, TransactionView};
use penumbra_tct::{Proof, StateCommitment, Tree};
use penumbra_transaction::plan::TransactionPlan;
use penumbra_transaction::{AuthorizationData, Transaction, WitnessData};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Error;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryInto;
use std::str::FromStr;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxInfoResponse {
    txp: TransactionPerspective,
    txv: TransactionView,
}

impl TxInfoResponse {
    pub fn new(txp: TransactionPerspective, txv: TransactionView) -> TxInfoResponse {
        Self { txp, txv }
    }
}

/// encode transaction to bytes
/// Arguments:
///     transaction: `penumbra_transaction::Transaction`
/// Returns: `<Vec<u8>`
#[wasm_bindgen]
pub fn encode_tx(transaction: JsValue) -> WasmResult<JsValue> {
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
    let tx_vec: Vec<u8> =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, tx_bytes)?;
    let transaction: Transaction = Transaction::try_from(tx_vec)?;
    let result = serde_wasm_bindgen::to_value(&transaction)?;
    Ok(result)
}

/// TODO: Deprecated. Still used in `penumbra-zone/wallet`, remove when migration is complete.
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
) -> Result<JsValue, Error> {
    let plan: TransactionPlan = serde_wasm_bindgen::from_value(transaction_plan)?;

    let fvk = FullViewingKey::from_str(full_viewing_key.as_ref())
        .expect("The provided string is not a valid FullViewingKey");

    let auth_data = sign_plan(spend_key_str, plan.clone())?;

    let stored_tree: StoredTree =
        serde_wasm_bindgen::from_value(stored_tree).expect("able to parse StoredTree from JS");

    let nct = load_tree(stored_tree);

    let witness_data = witness(nct, plan.clone())?;

    let tx = build_transaction(&fvk, plan.clone(), auth_data, witness_data)?;

    serde_wasm_bindgen::to_value(&tx)
}

/// Get transaction view, transaction perspective
/// Arguments:
///     full_viewing_key: `bech32 String`
///     tx: `pbt::Transaction`
/// Returns: `TxInfoResponse`
#[wasm_bindgen]
pub async fn transaction_info(full_viewing_key: &str, tx: JsValue) -> Result<JsValue, Error> {
    let transaction = serde_wasm_bindgen::from_value(tx)?;
    let response = transaction_info_inner(full_viewing_key, transaction).await?;

    serde_wasm_bindgen::to_value(&response)
}

/// deprecated
pub async fn transaction_info_inner(
    full_viewing_key: &str,
    tx: Transaction,
) -> WasmResult<TxInfoResponse> {
    let storage = IndexedDBStorage::new().await?;

    let fvk = FullViewingKey::from_str(full_viewing_key)?;

    // First, create a TxP with the payload keys visible to our FVK and no other data.
    let mut txp = penumbra_transaction::TransactionPerspective {
        payload_keys: tx
            .payload_keys(&fvk)
            .expect("Error generating payload keys"),
        ..Default::default()
    };

    // Next, extend the TxP with the openings of commitments known to our view server
    // but not included in the transaction body, for instance spent notes or swap claim outputs.
    for action in tx.actions() {
        use penumbra_transaction::Action;
        match action {
            Action::Spend(spend) => {
                let nullifier = spend.body.nullifier;
                // An error here indicates we don't know the nullifier, so we omit it from the Perspective.
                if let Some(spendable_note_record) =
                    storage.get_note_by_nullifier(&nullifier).await?
                {
                    txp.spend_nullifiers
                        .insert(nullifier, spendable_note_record.note.clone());
                }
            }
            Action::SwapClaim(claim) => {
                let output_1_record = storage
                    .get_note(&claim.body.output_1_commitment)
                    .await?
                    .expect("Error generating TxP: SwapClaim output 1 commitment not found");

                let output_2_record = storage
                    .get_note(&claim.body.output_2_commitment)
                    .await?
                    .expect("Error generating TxP: SwapClaim output 2 commitment not found");

                txp.advice_notes
                    .insert(claim.body.output_1_commitment, output_1_record.note.clone());
                txp.advice_notes
                    .insert(claim.body.output_2_commitment, output_2_record.note.clone());
            }
            _ => {}
        }
    }

    // Now, generate a stub TxV from our minimal TxP, and inspect it to see what data we should
    // augment the minimal TxP with to provide additional context (e.g., filling in denoms for
    // visible asset IDs).
    let min_view = tx.view_from_perspective(&txp);
    let mut address_views = BTreeMap::new();
    let mut asset_ids = BTreeSet::new();
    for action_view in min_view.action_views() {
        use penumbra_dex::{swap::SwapView, swap_claim::SwapClaimView};
        use penumbra_transaction::view::action_view::{
            ActionView, DelegatorVoteView, OutputView, SpendView,
        };
        match action_view {
            ActionView::Spend(SpendView::Visible { note, .. }) => {
                let address = note.address();
                address_views.insert(address, fvk.view_address(address));
                asset_ids.insert(note.asset_id());
            }
            ActionView::Output(OutputView::Visible { note, .. }) => {
                let address = note.address();
                address_views.insert(address, fvk.view_address(address));
                asset_ids.insert(note.asset_id());
            }
            ActionView::Swap(SwapView::Visible { swap_plaintext, .. }) => {
                let address = swap_plaintext.claim_address;
                address_views.insert(address, fvk.view_address(address));
                asset_ids.insert(swap_plaintext.trading_pair.asset_1());
                asset_ids.insert(swap_plaintext.trading_pair.asset_2());
            }
            ActionView::SwapClaim(SwapClaimView::Visible {
                output_1, output_2, ..
            }) => {
                // Both will be sent to the same address so this only needs to be added once
                let address = output_1.address();
                address_views.insert(address, fvk.view_address(address));
                asset_ids.insert(output_1.asset_id());
                asset_ids.insert(output_2.asset_id());
            }
            ActionView::DelegatorVote(DelegatorVoteView::Visible { note, .. }) => {
                let address = note.address();
                address_views.insert(address, fvk.view_address(address));
                asset_ids.insert(note.asset_id());
            }
            _ => {}
        }
    }

    // Now, extend the TxP with information helpful to understand the data it can view:

    let mut denoms = Vec::new();

    for id in asset_ids {
        if let Some(denom) = storage.get_asset(&id).await? {
            denoms.push(denom.clone());
        }
    }

    txp.denoms.extend(denoms.into_iter());

    txp.address_views = address_views.into_values().collect();

    // Finally, compute the full TxV from the full TxP:
    let txv = tx.view_from_perspective(&txp);

    let txp_proto = TransactionPerspective::try_from(txp)?;
    let txv_proto = TransactionView::try_from(txv)?;

    let response = TxInfoResponse {
        txp: txp_proto,
        txv: txv_proto,
    };
    Ok(response)
}

fn sign_plan(
    spend_key_str: &str,
    transaction_plan: TransactionPlan,
) -> WasmResult<AuthorizationData> {
    let spend_key = SpendKey::from_str(spend_key_str)?;
    let auth_data = transaction_plan.authorize(OsRng, &spend_key);
    Ok(auth_data)
}

fn build_transaction(
    fvk: &FullViewingKey,
    plan: TransactionPlan,
    auth_data: AuthorizationData,
    witness_data: WitnessData,
) -> WasmResult<Transaction> {
    let tx = plan
        .build(fvk, witness_data)?
        .authorize(&mut OsRng, &auth_data)?;

    Ok(tx)
}

fn witness(nct: Tree, plan: TransactionPlan) -> WasmResult<WitnessData> {
    let note_commitments: Vec<StateCommitment> = plan
        .spend_plans()
        .filter(|plan| plan.note.amount() != 0u64.into())
        .map(|spend| spend.note.commit().into())
        .chain(
            plan.swap_claim_plans()
                .map(|swap_claim| swap_claim.swap_plaintext.swap_commitment()),
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
    Ok(witness_data)
}
