extern crate penumbra_wasm;

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde::{Deserialize, Serialize};
    use serde_json;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
    use indexed_db_futures::prelude::{
        IdbDatabase, IdbObjectStore, IdbQuerySource, IdbTransaction, IdbTransactionMode,
    };

    use penumbra_proto::{
        core::{
            asset::v1alpha1::Value,
            component::chain::v1alpha1::{ChainParameters, FmdParameters},
            keys::v1alpha1::{Address, AddressIndex},
            transaction::v1alpha1::{MemoPlaintext, TransactionPlan as tp},
        },
        view::v1alpha1::SpendableNoteRecord,
    };
    use penumbra_tct::{structure::Hash, Forgotten};
    use penumbra_transaction::{
        plan::{ActionPlan, TransactionPlan},
        Action,
    };
    use penumbra_wasm::{
        build::build_action,
        error::WasmError,
        keys::load_proving_key,
        storage::IndexedDBStorage,
        tx::{authorize, build, build_parallel, witness},
        wasm_planner::WasmPlanner,
    };

    #[wasm_bindgen_test]
    async fn mock_build_serial_and_parallel() {
        // Limit the use of Penumbra Rust libraries since we're mocking JS calls
        // that are based on constructing objects according to protobuf definitions.

        // Load the proving key parameters as byte arrays.
        let spend_key: &[u8] = include_bytes!("../../crypto/proof-params/src/gen/spend_pk.bin");
        let output_key: &[u8] = include_bytes!("../../crypto/proof-params/src/gen/output_pk.bin");
        let delegator_vote_key: &[u8] =
            include_bytes!("../../crypto/proof-params/src/gen/delegator_vote_pk.bin");
        let nullifier_derivation_key: &[u8] =
            include_bytes!("../../crypto/proof-params/src/gen/nullifier_derivation_pk.bin");
        let swap_key: &[u8] = include_bytes!("../../crypto/proof-params/src/gen/swap_pk.bin");
        let swap_claim_key: &[u8] =
            include_bytes!("../../crypto/proof-params/src/gen/swapclaim_pk.bin");
        let convert_key: &[u8] = include_bytes!("../../crypto/proof-params/src/gen/convert_pk.bin");

        // Serialize &[u8] to JsValue.
        let spend_key_js: JsValue = serde_wasm_bindgen::to_value(&spend_key).unwrap();
        let output_key_js: JsValue = serde_wasm_bindgen::to_value(&output_key).unwrap();
        let delegator_vote_key_js: JsValue =
            serde_wasm_bindgen::to_value(&delegator_vote_key).unwrap();
        let nullifier_derivation_key_js: JsValue =
            serde_wasm_bindgen::to_value(&nullifier_derivation_key).unwrap();
        let swap_key_js: JsValue = serde_wasm_bindgen::to_value(&swap_key).unwrap();
        let swap_claim_key_js: JsValue = serde_wasm_bindgen::to_value(&swap_claim_key).unwrap();
        let convert_key_js: JsValue = serde_wasm_bindgen::to_value(&convert_key).unwrap();

        // Dynamically load the proving keys at runtime for each key type.
        load_proving_key(spend_key_js, "spend").expect("can load spend key");
        load_proving_key(output_key_js, "output").expect("can load output key");
        load_proving_key(delegator_vote_key_js, "delegator_vote")
            .expect("can load delegator vote key");
        load_proving_key(nullifier_derivation_key_js, "nullifier_derivation")
            .expect("can load nullifier derivation key");
        load_proving_key(swap_key_js, "swap").expect("can load swap key");
        load_proving_key(swap_claim_key_js, "swap_claim").expect("can load swap claim key");
        load_proving_key(convert_key_js, "convert").expect("can load convert key");

        // Define database parameters.
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct IndexedDbConstants {
            name: String,
            version: u32,
            tables: Tables,
        }

        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct Tables {
            assets: String,
            notes: String,
            spendable_notes: String,
            swaps: String,
        }

        // Define `IndexDB` table parameters and constants.
        let tables: Tables = Tables {
            assets: "ASSETS".to_string(),
            notes: "NOTES".to_string(),
            spendable_notes: "SPENDABLE_NOTES".to_string(),
            swaps: "SWAPS".to_string(),
        };

        let constants: IndexedDbConstants = IndexedDbConstants {
            name: "penumbra-db-wasm-test".to_string(),
            version: 1,
            tables,
        };

        // Sample chain and fmd parameters.
        let chain_params = ChainParameters {
            chain_id: "penumbra-testnet-iapetus".to_string(),
            epoch_duration: 5u64,
        };

        let fmd_params = FmdParameters {
            precision_bits: 0u32,
            as_of_block_height: 1u64,
        };

        // Serialize the parameters into `JsValue`.
        let js_chain_params_value: JsValue = serde_wasm_bindgen::to_value(&chain_params).unwrap();
        let js_fmd_params_value: JsValue = serde_wasm_bindgen::to_value(&fmd_params).unwrap();
        let js_constants_params_value: JsValue = serde_wasm_bindgen::to_value(&constants).unwrap();

        // Construct `WasmPlanner` instance.
        let mut wasm_planner = WasmPlanner::new(
            js_constants_params_value,
            js_chain_params_value,
            js_fmd_params_value,
        )
        .await
        .unwrap();

        // Create spendable UTXO note in JSON format.
        let spendable_note_json = r#"
        {
            "note_commitment": {
                "inner": "MY7PmcrH4fhjFOoMIKEdF+x9EUhZ9CS/CIfVco7Y5wU="
            },
            "note": {
                "value": {
                    "amount": {
                        "lo": "1000000",
                        "hi": "0"
                    },
                    "asset_id": {
                        "inner": "nwPDkQq3OvLnBwGTD+nmv1Ifb2GEmFCgNHrU++9BsRE=",
                        "alt_bech32m": "",
                        "alt_base_denom": ""
                    }
                },
                "rseed": "p2w4O1ognDJtKVqhHK2qsUbV+1AEM/gn58uWYQ5v3sM=",
                "address": {
                    "inner": "F6T1P51M1QOu8NGhKTMdJTy72TDhB2h00uvlIUcXVdovybq4ZcOwROB+1VE/ar4thEDNPanAcaYOrL+FugN8e19pvr93ZqmTjUdOLic+w+U=",
                    "alt_bech32m": ""
                }
            },
            "address_index": {
                "account": "0",
                "randomizer": "AAAAAAAAAAAAAAAA"
            },
            "nullifier": {
                "inner": "8TvyFVKk16PHcOEAgl0QV4/92xdVpLdXI+zP87lBrQ8="
            },
            "height_created": "250305",
            "height_spent": "0",
            "position": "3204061134848",
            "source": {
                "transaction": {
                    "id": "oJ9Bo9v22srtUmKdTAMVwPOuGumWE2cAuBbZHci8B1I="
                }
            }
        }
        "#;

        // Convert note to `SpendableNoteRecord`.
        let spendable_note: SpendableNoteRecord =
            serde_json::from_str(spendable_note_json).unwrap();

        // Define neccessary parameters to mock `TransactionPlannerRequest` in JSON format.
        let address_json = r#"
        {
            "alt_bech32m": "penumbra1dugkjttfezh4gfkqs77377gnjlvmkkehusx6953udxeescc0qpgk6gqc0jmrsjq8xphzrg938843p0e63z09vt8lzzmef0q330e5njuwh4290n8pemcmx70sasym0lcjkstgzc",
            "inner": ""
        }
        "#;
        let value_json = r#"
        {
            "amount": {
                "lo": "1",
                "hi": "0"
            },
            "asset_id": { 
                "inner": "nwPDkQq3OvLnBwGTD+nmv1Ifb2GEmFCgNHrU++9BsRE=", 
                "alt_bech32m": "", 
                "alt_base_denom": "" 
            }
        }
        "#;

        // Convert fields to JsValue.
        let address: Address = serde_json::from_str(address_json).unwrap();
        let value: Value = serde_json::from_str(value_json).unwrap();

        // Add output action to plan.
        wasm_planner
            .output(
                serde_wasm_bindgen::to_value(&value).unwrap(),
                serde_wasm_bindgen::to_value(&address).unwrap(),
            )
            .unwrap();

        // Add memo to plan.
        let memo: MemoPlaintext = MemoPlaintext {
            return_address: Some(address),
            text: "sample memo".to_string(),
        };
        let memo_plan_deserialized = serde_wasm_bindgen::to_value(&memo).unwrap();
        wasm_planner.memo(memo_plan_deserialized).unwrap();

        // Retrieve private database handle with public getters.
        let storage = wasm_planner.get_storage();
        let storage_ref: &IndexedDBStorage = unsafe { &*storage };
        let database: *const IdbDatabase = storage_ref.get_database();
        let database_ref: &IdbDatabase = unsafe { &*database };

        // Define SCT-related structs.
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct Position {
            epoch: u64,
            block: u64,
            commitment: u64,
        }

        #[derive(Clone, Debug, Serialize, Deserialize)]
        #[allow(non_snake_case)]
        pub struct StoredPosition {
            Position: Position,
        }

        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct StoreHash {
            position: Position,
            height: u64,
            hash: Hash,
            essential: bool,
        }

        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct StoreCommitment {
            commitment: Commitment,
            position: Position,
        }

        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct Commitment {
            inner: String,
        }

        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct StateCommitmentTree {
            last_position: Position,
            last_forgotten: u64,
            hashes: StoreHash,
            commitments: StoreCommitment,
        }

        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct SctUpdates {
            store_commitments: StoreCommitment,
            set_position: StoredPosition,
            set_forgotten: u64,
        }

        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct StoredTree {
            last_position: Option<StoredPosition>,
            last_forgotten: Option<Forgotten>,
            hashes: Vec<StoreHash>,
            commitments: Vec<StoreCommitment>,
        }

        // Define a sample SCT update.
        #[allow(non_snake_case)]
        let sctUpdates = SctUpdates {
            store_commitments: StoreCommitment {
                commitment: Commitment {
                    inner: "MY7PmcrH4fhjFOoMIKEdF+x9EUhZ9CS/CIfVco7Y5wU=".to_string(),
                },
                position: Position {
                    epoch: 746u64,
                    block: 237u64,
                    commitment: 0u64,
                },
            },
            set_position: StoredPosition {
                Position: Position {
                    epoch: 750u64,
                    block: 710u64,
                    commitment: 0u64,
                },
            },
            set_forgotten: 3u64,
        };

        // Populate database with records (CRUD).
        let tx_note: IdbTransaction = database_ref
            .transaction_on_one_with_mode("SPENDABLE_NOTES", IdbTransactionMode::Readwrite)
            .unwrap();
        let tx_tree_commitments: IdbTransaction = database_ref
            .transaction_on_one_with_mode("TREE_COMMITMENTS", IdbTransactionMode::Readwrite)
            .unwrap();
        let tx_tree_last_position: IdbTransaction = database_ref
            .transaction_on_one_with_mode("TREE_LAST_POSITION", IdbTransactionMode::Readwrite)
            .unwrap();
        let tx_tree_last_forgotten: IdbTransaction = database_ref
            .transaction_on_one_with_mode("TREE_LAST_FORGOTTEN", IdbTransactionMode::Readwrite)
            .unwrap();

        let store_note: IdbObjectStore = tx_note.object_store("SPENDABLE_NOTES").unwrap();
        let store_tree_commitments: IdbObjectStore = tx_tree_commitments
            .object_store("TREE_COMMITMENTS")
            .unwrap();
        let store_tree_last_position: IdbObjectStore = tx_tree_last_position
            .object_store("TREE_LAST_POSITION")
            .unwrap();
        let store_tree_last_forgotten: IdbObjectStore = tx_tree_last_forgotten
            .object_store("TREE_LAST_FORGOTTEN")
            .unwrap();

        let spendable_note_json = serde_wasm_bindgen::to_value(&spendable_note).unwrap();
        let tree_commitments_json =
            serde_wasm_bindgen::to_value(&sctUpdates.store_commitments).unwrap();
        let tree_position_json_value =
            serde_wasm_bindgen::to_value(&sctUpdates.set_position).unwrap();
        let tree_position_json_key = serde_wasm_bindgen::to_value(&"last_position").unwrap();
        let tree_last_forgotten_json_value =
            serde_wasm_bindgen::to_value(&sctUpdates.set_forgotten).unwrap();
        let tree_last_forgotten_json_key: JsValue =
            serde_wasm_bindgen::to_value(&"last_forgotten").unwrap();

        store_note.put_val(&spendable_note_json).unwrap();
        store_tree_commitments
            .put_val(&tree_commitments_json)
            .unwrap();
        store_tree_last_position
            .put_key_val(&tree_position_json_key, &tree_position_json_value)
            .unwrap();
        store_tree_last_forgotten
            .put_key_val(
                &tree_last_forgotten_json_key,
                &tree_last_forgotten_json_value,
            )
            .unwrap();

        // Set refund address.
        #[derive(Clone, Debug, Serialize, Deserialize)]
        struct RefundAddress {
            inner: String,
        }
        let refund_address = RefundAddress {
            inner: "ts1I61pd5+xWqlwcuPwsPOGbjevxAoQVymTXyHe60jLlY57WHcAuGsSwYuSxnOX+nTgEBm3MHn7mBlNTxqEkbnJwlNu6YUSDmA8D+aOqCT4=".to_string(),
        };
        let refund_address_json: JsValue = serde_wasm_bindgen::to_value(&refund_address).unwrap();
        let source: JsValue = serde_wasm_bindgen::to_value(&None::<AddressIndex>).unwrap();

        // -------------- 1. Query transaction plan performing a spend --------------

        let transaction_plan: JsValue = wasm_planner
            .plan(refund_address_json, source)
            .await
            .unwrap();

        // -------------- 2. Generate authorization data from spend key and transaction plan --------------

        let spend_key =
            "penumbraspendkey1qul0huewkcmemljd5m3vz3awqt7442tjg2dudahvzu6eyj9qf0eszrnguh"
                .to_string();

        let authorization_data = authorize(&spend_key, transaction_plan.clone()).unwrap();

        // -------------- 3. Generate witness --------------

        // Retrieve SCT from storage.
        let tx_last_position: IdbTransaction<'_> = database_ref
            .transaction_on_one("TREE_LAST_POSITION")
            .unwrap();
        let store_last_position = tx_last_position.object_store("TREE_LAST_POSITION").unwrap();
        let value_last_position: Option<JsValue> = store_last_position
            .get_owned("last_position")
            .unwrap()
            .await
            .unwrap();

        let tx_last_forgotten = database_ref
            .transaction_on_one("TREE_LAST_FORGOTTEN")
            .unwrap();
        let store_last_forgotten = tx_last_forgotten
            .object_store("TREE_LAST_FORGOTTEN")
            .unwrap();
        let value_last_forgotten: Option<JsValue> = store_last_forgotten
            .get_owned("last_forgotten")
            .unwrap()
            .await
            .unwrap();

        let tx_commitments = database_ref.transaction_on_one("TREE_COMMITMENTS").unwrap();
        let store_commitments = tx_commitments.object_store("TREE_COMMITMENTS").unwrap();
        let value_commitments = store_commitments
            .get_owned("MY7PmcrH4fhjFOoMIKEdF+x9EUhZ9CS/CIfVco7Y5wU=")
            .unwrap()
            .await
            .unwrap();

        // Convert retrieved values to `JsValue`.
        let last_position_json: StoredPosition =
            serde_wasm_bindgen::from_value(value_last_position.unwrap()).unwrap();
        let last_forgotten_json: Forgotten =
            serde_wasm_bindgen::from_value(value_last_forgotten.unwrap()).unwrap();
        let commitments_jsvalue: StoreCommitment =
            serde_wasm_bindgen::from_value(JsValue::from(value_commitments.clone())).unwrap();

        // Reconstruct SCT struct.
        let mut vec_store_commitments: Vec<StoreCommitment> = Vec::new();
        vec_store_commitments.push(commitments_jsvalue.clone());

        let sct = StoredTree {
            last_position: Some(last_position_json.clone()),
            last_forgotten: Some(last_forgotten_json.clone()),
            hashes: [].to_vec(),
            commitments: vec_store_commitments,
        };

        // Convert SCT to `JsValue`.
        let sct_json = serde_wasm_bindgen::to_value(&sct).unwrap();

        // Generate witness data from SCT and specific transaction plan.
        let witness_data: Result<JsValue, WasmError> = witness(transaction_plan.clone(), sct_json);

        // Viewing key to reveal asset balances and transactions.
        let full_viewing_key = "penumbrafullviewingkey1mnm04x7yx5tyznswlp0sxs8nsxtgxr9p98dp0msuek8fzxuknuzawjpct8zdevcvm3tsph0wvsuw33x2q42e7sf29q904hwerma8xzgrxsgq2";

        // Serialize transaction plan into `TransactionPlan`.
        let transaction_plan_serialized: tp =
            serde_wasm_bindgen::from_value(transaction_plan.clone()).unwrap();
        let transaction_plan_conv: TransactionPlan =
            transaction_plan_serialized.try_into().unwrap();

        // -------------- 4. Build the (1) Serial Transaction and (2) Parallel Transaction --------------

        let mut actions: Vec<Action> = Vec::new();

        for i in transaction_plan_conv.actions.clone() {
            if let ActionPlan::Spend(ref _spend_plan) = i {
                let action_deserialize = serde_wasm_bindgen::to_value(&i).unwrap();
                let action = build_action(
                    transaction_plan.clone(),
                    action_deserialize,
                    full_viewing_key,
                    witness_data.as_ref().unwrap().clone(),
                )
                .unwrap();
                let action_serialize: Action =
                    serde_wasm_bindgen::from_value(action.clone()).unwrap();
                actions.push(action_serialize);
            }
            if let ActionPlan::Output(ref _output_plan) = i {
                let action_deserialize = serde_wasm_bindgen::to_value(&i).unwrap();
                let action = build_action(
                    transaction_plan.clone(),
                    action_deserialize,
                    full_viewing_key,
                    witness_data.as_ref().unwrap().clone(),
                )
                .unwrap();
                let action_serialize: Action =
                    serde_wasm_bindgen::from_value(action.clone()).unwrap();
                actions.push(action_serialize);
            }
        }

        // Deserialize actions.
        let action_deserialized = serde_wasm_bindgen::to_value(&actions).unwrap();

        // Execute parallel spend transaction and generate proof.
        let parallel_transaction = build_parallel(
            action_deserialized,
            transaction_plan.clone(),
            witness_data.as_ref().unwrap().clone(),
            authorization_data.clone(),
        )
        .unwrap();
        console_log!("Parallel transaction is: {:?}", parallel_transaction);

        // Execute serial spend transaction and generate proof.
        let serial_transaction = build(
            full_viewing_key,
            transaction_plan.clone(),
            witness_data.as_ref().unwrap().clone(),
            authorization_data.clone(),
        )
        .unwrap();
        console_log!("Serial transaction is: {:?}", serial_transaction);
    }
}
