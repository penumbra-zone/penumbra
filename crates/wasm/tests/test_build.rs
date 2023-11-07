extern crate penumbra_wasm;

#[cfg(test)]
mod tests {
    use std::vec;
    use indexed_db_futures::prelude::{IdbObjectStore, IdbTransaction, IdbTransactionMode};
    use indexed_db_futures::prelude::{OpenDbRequest, IdbObjectStoreParameters};
    use indexed_db_futures::request::IdbOpenDbRequestLike;
    use indexed_db_futures::{prelude, IdbVersionChangeEvent}; 
    use penumbra_asset::asset::Id;
    use penumbra_asset::balance::commitment;
    use penumbra_proto::core::component::shielded_pool::v1alpha1::Spend;
    use penumbra_proto::serializers::bech32str::full_viewing_key;
    use penumbra_tct::Forgotten;
    use penumbra_tct::storage::StoreCommitment;
    use penumbra_wasm::tx::authorize;
    // use penumbra_wasm::tx::build;
    use penumbra_wasm::build::{self, build_parallel};
    use penumbra_wasm::tx::witness;
    use penumbra_wasm::view_server::StoredTree;
    use penumbra_wasm::wasm_planner;
    use wasm_bindgen_test::*;
    use serde::{Serialize, Deserialize};
    use serde_json::json;
    extern crate serde;
    use serde_json;
    use wasm_bindgen::JsValue;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
    use penumbra_proto::core::asset::v1alpha1::AssetId;
    use penumbra_proto::view::v1alpha1::TransactionPlannerRequest;
    use penumbra_proto::view::v1alpha1::transaction_planner_request as tpr;
    use penumbra_proto::core::keys::v1alpha1::Address;
    use penumbra_proto::core::asset::v1alpha1::Value;
    use penumbra_proto::core::num::v1alpha1::Amount;
    use web_sys::console;
    use crate::penumbra_wasm::storage::{IndexedDBStorage, IndexedDbConstants, Tables};
    use crate::penumbra_wasm::error::{WasmResult, WasmError};
    use indexed_db_futures::{IdbDatabase, IdbQuerySource, IdbKeyPath};
    use penumbra_wasm::utils;
    use penumbra_proto::core::component::chain::v1alpha1::ChainParameters;
    use penumbra_proto::core::component::chain::v1alpha1::FmdParameters;
    use penumbra_proto::core::keys::v1alpha1::AddressIndex;
    use crate::penumbra_wasm::wasm_planner::WasmPlanner;
    use penumbra_proto::core::component::sct::v1alpha1::Nullifier;
    use penumbra_proto::core::component::chain::v1alpha1::NoteSource;
    use penumbra_proto::view::v1alpha1::SpendableNoteRecord;
    use penumbra_proto::crypto::tct::v1alpha1::StateCommitment;
    use penumbra_proto::core::transaction::v1alpha1::TransactionPlan;
    use penumbra_proto::penumbra::core::num::v1alpha1 as pb;
    use penumbra_proto::core::component::shielded_pool::v1alpha1::Note;
    use js_sys::Array;
    use serde_json::from_value;
    use penumbra_tct::structure::Hash;
    use penumbra_proto::core::transaction::v1alpha1 as ps;
    use penumbra_transaction::AuthorizationData;
    use penumbra_proto::DomainType;
    use rand_core::OsRng;

    #[wasm_bindgen_test]
    async fn mock_build() {
        // Limit the use of Penumbra Rust libraries since we're mocking JS calls
        // based on constructing objects according to protobuf definitions.
    
        // Sample chain and fmd parameters.
        let chain_params = ChainParameters {
            chain_id: "penumbra-testnet-iapetus".to_string(),
            epoch_duration: 5u64,
        };

        let fmd_params =  FmdParameters {
            precision_bits: 0u32,
            as_of_block_height: 1u64,
        }; 

        // IndexDB tables and constants.
        let tables: Tables = Tables { 
            assets: "ASSETS".to_string(), 
            notes: "NOTES".to_string(), 
            spendable_notes: "SPENDABLE_NOTES".to_string(), 
            swaps: "SWAPS".to_string(),
        };

        let constants: IndexedDbConstants = IndexedDbConstants { 
            name: "penumbra-db-wasm".to_string(), 
            version: 1, 
            tables,
        };

        // Serialize the parameters into `JsValue`.
        let js_chain_params_value: JsValue = serde_wasm_bindgen::to_value(&chain_params).unwrap();
        let js_fmd_params_value: JsValue = serde_wasm_bindgen::to_value(&fmd_params).unwrap();
        let js_constants_params_value: JsValue = serde_wasm_bindgen::to_value(&constants).unwrap();

        // Construct `WasmPlanner` instance.
        let mut wasm_planner = WasmPlanner::new(
            js_constants_params_value, 
            js_chain_params_value, 
            js_fmd_params_value
        ).await.unwrap();

        // Create spendable UTXO note.
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
                "inner": "oJ9Bo9v22srtUmKdTAMVwPOuGumWE2cAuBbZHci8B1I="
            }
        }
        "#;

        // Convert note to `SpendableNoteRecord`.
        let spendable_note: SpendableNoteRecord = serde_json::from_str(spendable_note_json).unwrap();

        // Define neccessary parameters to mock `TransactionPlannerRequest`.
        let address_json = r#"
        {
            "alt_bech32m": "penumbrav2t1ztjrnr9974u4308zxy3sc378sh0k2r8mh0xqt9525c78l9vlyxf2w7c087tlzp4pnk9a7ztvlrnp9lf7hqx3wsm9su4e7vchtav0ap3lpnedry5hfn22hnu9vvaxjpv0t8phvp",
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
        let address_original: Address = serde_json::from_str(address_json).unwrap();
        let value_original: Value = serde_json::from_str(value_json).unwrap();

        // Add output action to plan.
        wasm_planner.output(
            serde_wasm_bindgen::to_value(&value_original).unwrap(), 
            serde_wasm_bindgen::to_value(&address_original).unwrap()
        );

        // Retrieve database handle.
        let storage = wasm_planner.get_storage();
        let storage_ref: &IndexedDBStorage = unsafe { &*storage };
        let database = storage_ref.get_database();
        let database_ref: &IdbDatabase = unsafe {&*database };
        
        // Define SCT structure. 
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct Position {
            epoch: u64,
            block: u64,
            commitment: u64,
        };  
        
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct StoredPosition { Position: Position }
        
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
        };

        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct StoredTree {
            last_position: Option<StoredPosition>,
            last_forgotten: Option<Forgotten>,
            hashes: Vec<StoreHash>,
            commitments: Vec<StoreCommitment>,
        }

        let sctUpdates = SctUpdates {
            store_commitments: StoreCommitment {
                commitment: Commitment { 
                    inner: "MY7PmcrH4fhjFOoMIKEdF+x9EUhZ9CS/CIfVco7Y5wU=".to_string() 
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
        let tx_note: IdbTransaction = database_ref.transaction_on_one_with_mode("SPENDABLE_NOTES", IdbTransactionMode::Readwrite).unwrap();
        let tx_tree_commitments: IdbTransaction = database_ref.transaction_on_one_with_mode("TREE_COMMITMENTS", IdbTransactionMode::Readwrite).unwrap();
        let tx_tree_last_position: IdbTransaction = database_ref.transaction_on_one_with_mode("TREE_LAST_POSITION", IdbTransactionMode::Readwrite).unwrap();
        let tx_tree_last_forgotten: IdbTransaction = database_ref.transaction_on_one_with_mode("TREE_LAST_FORGOTTEN", IdbTransactionMode::Readwrite).unwrap();

        let store_note: IdbObjectStore = tx_note.object_store("SPENDABLE_NOTES").unwrap();
        let store_tree_commitments: IdbObjectStore = tx_tree_commitments.object_store("TREE_COMMITMENTS").unwrap();
        let store_tree_last_position: IdbObjectStore = tx_tree_last_position.object_store("TREE_LAST_POSITION").unwrap();
        let store_tree_last_forgotten: IdbObjectStore = tx_tree_last_forgotten.object_store("TREE_LAST_FORGOTTEN").unwrap();

        let spendable_note_json = serde_wasm_bindgen::to_value(&spendable_note).unwrap();
        let tree_commitments_json = serde_wasm_bindgen::to_value(&sctUpdates.store_commitments).unwrap();
        let tree_position_json_value = serde_wasm_bindgen::to_value(&sctUpdates.set_position).unwrap();
        let tree_position_json_key = serde_wasm_bindgen::to_value(&"last_position").unwrap();
        let tree_last_forgotten_json_value = serde_wasm_bindgen::to_value(&sctUpdates.set_forgotten).unwrap();
        let tree_last_forgotten_json_key: JsValue = serde_wasm_bindgen::to_value(&"last_forgotten").unwrap();

        store_note.put_val(&spendable_note_json);
        store_tree_commitments.put_val(&tree_commitments_json);
        store_tree_last_position.put_key_val(&tree_position_json_key, &tree_position_json_value);
        store_tree_last_forgotten.put_key_val(&tree_last_forgotten_json_key,&tree_last_forgotten_json_value);
                    
        // Set refund address.
        #[derive(Clone, Debug, Serialize, Deserialize)]
        struct RefundAddress {
            inner: String,
        }
        let refund_address = RefundAddress {
            inner: "ts1I61pd5+xWqlwcuPwsPOGbjevxAoQVymTXyHe60jLlY57WHcAuGsSwYuSxnOX+nTgEBm3MHn7mBlNTxqEkbnJwlNu6YUSDmA8D+aOqCT4=".to_string(),
        };
        let refund_address_json: JsValue = serde_wasm_bindgen::to_value(&refund_address).unwrap();

        // -------------- 1. Query transaction plan performing a spend -------------- 

        let transaction_plan = wasm_planner.plan(refund_address_json).await.unwrap();

        // -------------- 2. Generate authorization data from spend key and transaction plan -------------- 

        let spend_key = "penumbraspendkey1qul0huewkcmemljd5m3vz3awqt7442tjg2dudahvzu6eyj9qf0eszrnguh".to_string();
    
        let authorization_data = authorize(
            &spend_key, 
            transaction_plan.clone()
        ).unwrap();

        // -------------- 3. Generate witness and build the planned transaction --------------

        // Retrieve SCT.
        let tx_last_position: IdbTransaction<'_> = database_ref.transaction_on_one("TREE_LAST_POSITION").unwrap();
        let store_last_position = tx_last_position.object_store("TREE_LAST_POSITION").unwrap();
        let value_last_position: Option<JsValue> = store_last_position.get_owned("last_position").unwrap().await.unwrap();

        let tx_last_forgotten = database_ref.transaction_on_one("TREE_LAST_FORGOTTEN").unwrap();
        let store_last_forgotten = tx_last_forgotten.object_store("TREE_LAST_FORGOTTEN").unwrap();
        let value_last_forgotten: Option<JsValue> = store_last_forgotten.get_owned("last_forgotten").unwrap().await.unwrap();

        let tx_commitments = database_ref.transaction_on_one("TREE_COMMITMENTS").unwrap();
        let store_commitments = tx_commitments.object_store("TREE_COMMITMENTS").unwrap();
        let value_commitments = store_commitments.get_owned("MY7PmcrH4fhjFOoMIKEdF+x9EUhZ9CS/CIfVco7Y5wU=").unwrap().await.unwrap();

        // Convert retrieved storage values to `JsValue`.
        let last_position_json: StoredPosition = serde_wasm_bindgen::from_value(value_last_position.unwrap()).unwrap();
        let last_forgotten_json: Forgotten = serde_wasm_bindgen::from_value(value_last_forgotten.unwrap()).unwrap();
        let commitments_jsvalue: StoreCommitment = serde_wasm_bindgen::from_value(JsValue::from(value_commitments.clone())).unwrap();

        // Reconstruct SCT.
        let mut vec_store_commitments: Vec<StoreCommitment> = Vec::new();
        vec_store_commitments.push(commitments_jsvalue.clone());

        let sct = StoredTree { 
            last_position: Some(last_position_json.clone()), 
            last_forgotten: Some(last_forgotten_json.clone()), 
            hashes: [].to_vec(), 
            commitments: vec_store_commitments
        };
        
        // Convert SCT to `JsValue`.
        let sct_json = serde_wasm_bindgen::to_value(&sct).unwrap();

        // Generate witness data from SCT and specific transaction plan.
        let witness_data: Result<JsValue, WasmError> = witness(transaction_plan.clone(), sct_json);

        // Viewing key to reveal asset balances and transactions.
        let full_viewing_key = "penumbrafullviewingkey1mnm04x7yx5tyznswlp0sxs8nsxtgxr9p98dp0msuek8fzxuknuzawjpct8zdevcvm3tsph0wvsuw33x2q42e7sf29q904hwerma8xzgrxsgq2";

        // Execute spend transaction and proof.
        let transaction = build_parallel(
            full_viewing_key, 
            transaction_plan, 
            witness_data.unwrap(), 
            authorization_data.clone()
        ).unwrap();

        console_log!("transaction is: {:?}", transaction);
    }
}