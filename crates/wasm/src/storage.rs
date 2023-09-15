use crate::error::{WasmError, WasmResult};
use crate::note_record::SpendableNoteRecord;
use indexed_db_futures::prelude::OpenDbRequest;
use indexed_db_futures::{IdbDatabase, IdbQuerySource};
use penumbra_asset::asset::{DenomMetadata, Id};
use penumbra_proto::view::v1alpha1::NotesRequest;
use penumbra_proto::DomainType;
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::{note, Note};

enum Store {
    SpendableNotesStore,
}

impl Store {
    fn as_str(&self) -> &'static str {
        match *self {
            // *self has type Direction
            Store::SpendableNotesStore => "spendable_notes",
        }
    }
}

pub struct IndexedDBStorage {
    db: IdbDatabase,
}

impl IndexedDBStorage {
    pub async fn new() -> WasmResult<Self> {
        let db_req: OpenDbRequest = IdbDatabase::open_u32("penumbra", 12)?;

        let db: IdbDatabase = db_req.into_future().await?;

        Ok(IndexedDBStorage { db })
    }

    pub async fn get_notes(&self, request: NotesRequest) -> WasmResult<Vec<SpendableNoteRecord>> {
        let idb_tx = self.db.transaction_on_one("spendable_notes")?;
        let store = idb_tx.object_store("spendable_notes")?;

        let values = store.get_all()?.await?;

        let notes: Vec<SpendableNoteRecord> = values
            .into_iter()
            .map(|js_value| serde_wasm_bindgen::from_value(js_value).ok())
            .filter_map(|note_option| {
                note_option.and_then(|note: SpendableNoteRecord| match request.asset_id.clone() {
                    Some(asset_id) => {
                        if note.note.asset_id() == asset_id.clone().try_into().unwrap()
                            && note.height_spent == None
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
        let db_req: OpenDbRequest = IdbDatabase::open_u32("penumbra", 12)?;
        let db: IdbDatabase = db_req.into_future().await?;

        let tx = db.transaction_on_one("assets")?;
        let store = tx.object_store("assets")?;

        Ok(store
            .get_owned(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                id.to_proto().inner,
            ))?
            .await?
            .map(|js_value| serde_wasm_bindgen::from_value(js_value))
            .transpose()
            .map_err(WasmError::from)?)
    }

    pub async fn get_note(
        &self,
        commitment: &note::StateCommitment,
    ) -> WasmResult<Option<SpendableNoteRecord>> {
        let db_req: OpenDbRequest = IdbDatabase::open_u32("penumbra", 12)?;

        let db: IdbDatabase = db_req.into_future().await?;

        let tx = db.transaction_on_one("spendable_notes")?;
        let store = tx.object_store("spendable_notes")?;

        Ok(store
            .get_owned(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                commitment.to_proto().inner,
            ))?
            .await?
            .map(|js_value| serde_wasm_bindgen::from_value(js_value))
            .transpose()
            .map_err(WasmError::from)?)
    }

    pub async fn get_note_by_nullifier(
        &self,
        nullifier: &Nullifier,
    ) -> WasmResult<Option<SpendableNoteRecord>> {
        let db_req: OpenDbRequest = IdbDatabase::open_u32("penumbra", 12)?;

        let db: IdbDatabase = db_req.into_future().await?;

        let tx = db.transaction_on_one("spendable_notes")?;
        let store = tx.object_store("spendable_notes")?;

        Ok(store
            .index("nullifier")?
            .get_owned(&base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                nullifier.to_proto().inner,
            ))?
            .await?
            .map(|js_value| serde_wasm_bindgen::from_value(js_value))
            .transpose()
            .map_err(WasmError::from)?)
    }

    pub async fn store_advice(&self, note: Note) -> WasmResult<()> {
        let db_req: OpenDbRequest = IdbDatabase::open_u32("penumbra", 12)?;

        let db: IdbDatabase = db_req.into_future().await?;

        let tx = db.transaction_on_one("notes")?;
        let store = tx.object_store("notes")?;

        let note_proto: penumbra_proto::core::crypto::v1alpha1::Note = note.clone().try_into()?;
        let note_js = serde_wasm_bindgen::to_value(&note_proto)?;

        let commitment_proto = note.commit().to_proto();

        let commitment_js = serde_wasm_bindgen::to_value(&commitment_proto)?;

        store.put_key_val_owned(commitment_js, &note_js)?;

        Ok(())
    }

    pub async fn read_advice(&self, commitment: note::StateCommitment) -> WasmResult<Option<Note>> {
        let db_req: OpenDbRequest = IdbDatabase::open_u32("penumbra", 12)?;

        let db: IdbDatabase = db_req.into_future().await?;

        let tx = db.transaction_on_one("notes")?;
        let store = tx.object_store("notes")?;

        let commitment_proto = commitment.to_proto();

        let commitment_js = serde_wasm_bindgen::to_value(&commitment_proto)?;

        Ok(store
            .get_owned(commitment_js)?
            .await?
            .map(|js_value| serde_wasm_bindgen::from_value(js_value))
            .transpose()
            .map_err(WasmError::from)?)
    }
}
