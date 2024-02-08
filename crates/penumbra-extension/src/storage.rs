#[allow(unused_imports)]
use std::future::IntoFuture;

use indexed_db_futures::{
    prelude::{IdbObjectStoreParameters, IdbOpenDbRequestLike, OpenDbRequest},
    IdbDatabase, IdbKeyPath, IdbQuerySource, IdbVersionChangeEvent,
};
use penumbra_asset::asset::{Id, Metadata};
use penumbra_proto::{
    crypto::tct::v1::StateCommitment,
    view::v1::{NotesRequest, SwapRecord},
    DomainType,
};
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::{note, Note};
use serde::{Deserialize, Serialize};
use web_sys::IdbTransactionMode::Readwrite;

use crate::error::{WasmError, WasmResult};
use crate::note_record::SpendableNoteRecord;
use wasm_bindgen::JsValue;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexedDbConstants {
    pub name: String,
    pub version: u32,
    pub tables: Tables,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tables {
    pub assets: String,
    pub notes: String,
    pub spendable_notes: String,
    pub swaps: String,
}

pub struct IndexedDBStorage {
    db: IdbDatabase,
    constants: IndexedDbConstants,
}

impl IndexedDBStorage {
    pub async fn new(constants: IndexedDbConstants) -> WasmResult<Self> {
        #[allow(unused_mut)]
        let mut db_req: OpenDbRequest = IdbDatabase::open_u32(&constants.name, constants.version)?;

        // Conditionally mock sample `IdbDatabase` database for testing purposes
        #[cfg(feature = "mock-database")]
        let db_req = IndexedDBStorage::mock_test_database(db_req)
            .into_future()
            .await;

        let db: IdbDatabase = db_req.into_future().await?;

        Ok(IndexedDBStorage { db, constants })
    }

    pub async fn mock_test_database(mut db_req: OpenDbRequest) -> OpenDbRequest {
        db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            // Check if the object store exists; create it if it doesn't
            if evt.db().name() == "penumbra-db-wasm-test" {
                let note_key: JsValue = serde_wasm_bindgen::to_value("noteCommitment.inner")?;
                let note_object_store_params = IdbObjectStoreParameters::new()
                    .key_path(Some(&IdbKeyPath::new(note_key)))
                    .to_owned();
                let note_object_store = evt.db().create_object_store_with_params(
                    "SPENDABLE_NOTES",
                    &note_object_store_params,
                )?;

                let nullifier_key: JsValue = serde_wasm_bindgen::to_value("nullifier.inner")?;
                note_object_store.create_index_with_params(
                    "nullifier",
                    &IdbKeyPath::new(nullifier_key),
                    web_sys::IdbIndexParameters::new().unique(false),
                )?;
                evt.db().create_object_store("TREE_LAST_POSITION")?;
                evt.db().create_object_store("TREE_LAST_FORGOTTEN")?;

                let commitment_key: JsValue = serde_wasm_bindgen::to_value("commitment.inner")?;
                let commitment_object_store_params = IdbObjectStoreParameters::new()
                    .key_path(Some(&IdbKeyPath::new(commitment_key)))
                    .to_owned();
                evt.db().create_object_store_with_params(
                    "TREE_COMMITMENTS",
                    &commitment_object_store_params,
                )?;
                evt.db().create_object_store("TREE_HASHES")?;
            }
            Ok(())
        }));

        db_req
    }

    pub fn get_database(&self) -> *const IdbDatabase {
        &self.db
    }

    pub async fn get_notes(&self, request: NotesRequest) -> WasmResult<Vec<SpendableNoteRecord>> {
        let idb_tx = self
            .db
            .transaction_on_one(&self.constants.tables.spendable_notes)?;
        let store = idb_tx.object_store(&self.constants.tables.spendable_notes)?;

        let raw_values = store.get_all()?.await?;
        let parsed_notes = raw_values
            .into_iter()
            .filter_map(|js_value| self.parse_note(js_value, &request).ok())
            .collect();

        Ok(parsed_notes)
    }

    fn parse_note(
        &self,
        js_value: JsValue,
        request: &NotesRequest,
    ) -> WasmResult<SpendableNoteRecord> {
        let note: SpendableNoteRecord = serde_wasm_bindgen::from_value(js_value)?;

        let asset_id_matches = match &request.asset_id {
            Some(asset_id) => note.note.asset_id() == asset_id.clone().try_into()?,
            None => true,
        };

        let address_index_matches = match &request.address_index {
            Some(address_index) => note.address_index.eq(&address_index.clone().try_into()?),
            None => true,
        };

        if asset_id_matches && address_index_matches && note.height_spent.is_none() {
            Ok(note)
        } else {
            Err(WasmError::Anyhow(anyhow::anyhow!(
                "Note does not match the request"
            )))
        }
    }

    pub async fn get_asset(&self, id: &Id) -> WasmResult<Option<Metadata>> {
        let tx = self.db.transaction_on_one(&self.constants.tables.assets)?;
        let store = tx.object_store(&self.constants.tables.assets)?;

        Ok(store
            .get_owned(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                id.to_proto().inner,
            ))?
            .await?
            .map(serde_wasm_bindgen::from_value)
            .transpose()?)
    }

    pub async fn get_note(
        &self,
        commitment: &note::StateCommitment,
    ) -> WasmResult<Option<SpendableNoteRecord>> {
        let tx = self
            .db
            .transaction_on_one(&self.constants.tables.spendable_notes)?;
        let store = tx.object_store(&self.constants.tables.spendable_notes)?;

        Ok(store
            .get_owned(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                commitment.to_proto().inner,
            ))?
            .await?
            .map(serde_wasm_bindgen::from_value)
            .transpose()?)
    }

    pub async fn get_note_by_nullifier(
        &self,
        nullifier: &Nullifier,
    ) -> WasmResult<Option<SpendableNoteRecord>> {
        let tx = self
            .db
            .transaction_on_one(&self.constants.tables.spendable_notes)?;
        let store = tx.object_store(&self.constants.tables.spendable_notes)?;

        Ok(store
            .index("nullifier")?
            .get_owned(&base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                nullifier.to_proto().inner,
            ))?
            .await?
            .map(serde_wasm_bindgen::from_value)
            .transpose()?)
    }

    pub async fn store_advice(&self, note: Note) -> WasmResult<()> {
        let tx = self
            .db
            .transaction_on_one_with_mode(&self.constants.tables.notes, Readwrite)?;
        let store = tx.object_store(&self.constants.tables.notes)?;

        let note_proto: penumbra_proto::core::component::shielded_pool::v1::Note =
            note.clone().try_into()?;
        let note_js = serde_wasm_bindgen::to_value(&note_proto)?;

        let commitment_proto = note.commit().to_proto();

        store.put_key_val_owned(
            base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                commitment_proto.inner,
            ),
            &note_js,
        )?;

        Ok(())
    }

    pub async fn read_advice(&self, commitment: note::StateCommitment) -> WasmResult<Option<Note>> {
        let tx = self.db.transaction_on_one(&self.constants.tables.notes)?;
        let store = tx.object_store(&self.constants.tables.notes)?;

        let commitment_proto = commitment.to_proto();

        Ok(store
            .get_owned(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                commitment_proto.inner,
            ))?
            .await?
            .map(serde_wasm_bindgen::from_value)
            .transpose()?)
    }

    pub async fn get_swap_by_commitment(
        &self,
        swap_commitment: StateCommitment,
    ) -> WasmResult<Option<SwapRecord>> {
        let tx = self.db.transaction_on_one(&self.constants.tables.swaps)?;
        let store = tx.object_store(&self.constants.tables.swaps)?;

        Ok(store
            .get_owned(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                swap_commitment.inner,
            ))?
            .await?
            .map(serde_wasm_bindgen::from_value)
            .transpose()?)
    }
}
