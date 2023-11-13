use indexed_db_futures::{
    prelude::{OpenDbRequest, IdbOpenDbRequestLike, IdbObjectStoreParameters},
    IdbDatabase, IdbQuerySource, IdbKeyPath, IdbVersionChangeEvent

};
use serde::{Deserialize, Serialize};
use web_sys::IdbTransactionMode::Readwrite;
use penumbra_asset::asset::{DenomMetadata, Id};
use penumbra_proto::{
    crypto::tct::v1alpha1::StateCommitment,
    view::v1alpha1::{NotesRequest, SwapRecord},
    DomainType,
};
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::{note, Note};
use crate::error::WasmResult;
use crate::note_record::SpendableNoteRecord;
use wasm_bindgen::JsValue;

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

pub struct IndexedDBStorage {
    db: IdbDatabase,
    constants: IndexedDbConstants,
}

impl IndexedDBStorage {
    pub async fn new(constants: IndexedDbConstants) -> WasmResult<Self> { 
        let mut db_req: OpenDbRequest = IdbDatabase::open_u32(&constants.name, constants.version).unwrap();

        // Conditionally create object stores in the `IdbDatabase` database for testing purposes
        db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            // Check if the object store exists; create it if it doesn't
            if evt.db().name() == "penumbra-db-wasm-test" {
                let note_key: JsValue = serde_wasm_bindgen::to_value("noteCommitment.inner").unwrap();
                let note_object_store_params = IdbObjectStoreParameters::new()
                    .key_path(Some(&IdbKeyPath::new(note_key))).to_owned();
                let note_object_store = evt.db().create_object_store_with_params("SPENDABLE_NOTES", &note_object_store_params).unwrap();

                let nullifier_key: JsValue = serde_wasm_bindgen::to_value("nullifier.inner").unwrap();
                note_object_store.create_index_with_params(
                    "nullifier",
                    &IdbKeyPath::new(nullifier_key),
                    web_sys::IdbIndexParameters::new().unique(false),
                ).unwrap();
                evt.db().create_object_store("TREE_LAST_POSITION").unwrap();
                evt.db().create_object_store("TREE_LAST_FORGOTTEN").unwrap();

                let commitment_key: JsValue = serde_wasm_bindgen::to_value("commitment.inner").unwrap();
                let commitment_object_store_params = IdbObjectStoreParameters::new()
                    .key_path(Some(&IdbKeyPath::new(commitment_key))).to_owned();
                evt.db().create_object_store_with_params("TREE_COMMITMENTS", &commitment_object_store_params).unwrap();
                evt.db().create_object_store("TREE_HASHES").unwrap();
            }
            Ok(())
        }));
            
        let db: IdbDatabase = db_req.into_future().await.unwrap();
        
        Ok(IndexedDBStorage { db, constants })
    }

    pub fn get_database(&self) -> *const IdbDatabase {
        &self.db
    }

    pub async fn get_notes(&self, request: NotesRequest) -> WasmResult<Vec<SpendableNoteRecord>> {
        let idb_tx = self
            .db
            .transaction_on_one(&self.constants.tables.spendable_notes)?;
        let store = idb_tx.object_store(&self.constants.tables.spendable_notes)?;

        let values = store.get_all()?.await?;

        let notes: Vec<SpendableNoteRecord> = values
            .into_iter()
            .map(|js_value| serde_wasm_bindgen::from_value(js_value).ok())
            .filter_map(|note_option| {
                note_option.and_then(|note: SpendableNoteRecord| match request.asset_id.clone() {
                    Some(asset_id) => {
                        if note.note.asset_id() == asset_id.try_into().expect("Invalid asset id")
                            && note.height_spent.is_none()
                        {
                            Some(note)
                        } else {
                            None
                        }
                    }
                    None => Some(note),
                })
            })
            .collect();

        Ok(notes)
    }

    pub async fn get_asset(&self, id: &Id) -> WasmResult<Option<DenomMetadata>> {
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
        // Do not insert advice for zero amounts, simply return Ok because this is fine
        if u128::from(note.amount()) == 0u128 {
            return Ok(());
        }

        let tx = self
            .db
            .transaction_on_one_with_mode(&self.constants.tables.notes, Readwrite)?;
        let store = tx.object_store(&self.constants.tables.notes)?;

        let note_proto: penumbra_proto::core::component::shielded_pool::v1alpha1::Note =
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
