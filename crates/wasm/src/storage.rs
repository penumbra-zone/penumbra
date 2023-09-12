use crate::note_record::SpendableNoteRecord;
use anyhow::{anyhow, Result};
use indexed_db_futures::prelude::OpenDbRequest;
use indexed_db_futures::{IdbDatabase, IdbQuerySource};
use penumbra_proto::view::v1alpha1::NotesRequest;

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
    pub async fn new() -> Result<Self> {
        let db_req: OpenDbRequest = IdbDatabase::open_u32("penumbra", 12)
            .map_err(|_| anyhow!("error create OpenDbRequest "))?;

        let db: IdbDatabase = db_req
            .into_future()
            .await
            .map_err(|_| anyhow!("error open IdbDatabase"))?;

        Ok(IndexedDBStorage { db })
    }

    pub async fn get_notes(&self, request: NotesRequest) -> Result<Vec<SpendableNoteRecord>> {
        let idb_tx = self
            .db
            .transaction_on_one(Store::SpendableNotesStore.as_str())
            .map_err(|_| anyhow!("error create IdbTransaction"))?;
        let store = idb_tx
            .object_store(Store::SpendableNotesStore.as_str())
            .map_err(|_| anyhow!("error create IdbObjectStore"))?;

        let values = store
            .get_all()
            .map_err(|_| anyhow!("Idb store error"))?
            .await
            .map_err(|_| anyhow!("Idb store error"))?;

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
}
