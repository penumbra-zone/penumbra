use crate::error::WasmResult;
use crate::note_record::SpendableNoteRecord;
use indexed_db_futures::prelude::OpenDbRequest;
use indexed_db_futures::{IdbDatabase, IdbQuerySource};
use penumbra_asset::asset::{DenomMetadata, Id};
use penumbra_proto::core::component::chain::v1alpha1::{ChainParameters, FmdParameters};
use penumbra_proto::crypto::tct::v1alpha1::StateCommitment;
use penumbra_proto::view::v1alpha1::{NotesRequest, SwapRecord};
use penumbra_proto::DomainType;
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::{note, Note};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct IndexedDbConstants {
    name: String,
    version: u32,

    assets: String,
    chain_parameters: String,
    fmd_parameters: String,
    last_block_synced: String,
    notes: String,
    spendable_notes: String,
    swaps: String,
    tree_commitments: String,
    tree_hashes: String,
    tree_last_forgotten: String,
    tree_last_position: String,
}

pub struct IndexedDBStorage {
    db: IdbDatabase,
    constants: IndexedDbConstants,
}

impl IndexedDBStorage {
    pub async fn new(constants: IndexedDbConstants) -> WasmResult<Self> {
        let db_req: OpenDbRequest = IdbDatabase::open_u32(&constants.name, constants.version)?;

        let db: IdbDatabase = db_req.into_future().await?;

        Ok(IndexedDBStorage { db, constants })
    }

    pub async fn get_notes(&self, request: NotesRequest) -> WasmResult<Vec<SpendableNoteRecord>> {
        let idb_tx = self
            .db
            .transaction_on_one(&self.constants.spendable_notes)?;
        let store = idb_tx.object_store(&self.constants.spendable_notes)?;

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
        let tx = self.db.transaction_on_one(&self.constants.assets)?;
        let store = tx.object_store(&self.constants.assets)?;

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
            .transaction_on_one(&self.constants.spendable_notes)?;
        let store = tx.object_store(&self.constants.spendable_notes)?;

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
            .transaction_on_one(&self.constants.spendable_notes)?;
        let store = tx.object_store(&self.constants.spendable_notes)?;

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
        let tx = self.db.transaction_on_one(&self.constants.notes)?;
        let store = tx.object_store(&self.constants.notes)?;

        let note_proto: penumbra_proto::core::component::shielded_pool::v1alpha1::Note =
            note.clone().try_into()?;
        let note_js = serde_wasm_bindgen::to_value(&note_proto)?;

        let commitment_proto = note.commit().to_proto();

        let commitment_js = serde_wasm_bindgen::to_value(&commitment_proto)?;

        store.put_key_val_owned(commitment_js, &note_js)?;

        Ok(())
    }

    pub async fn read_advice(&self, commitment: note::StateCommitment) -> WasmResult<Option<Note>> {
        let tx = self.db.transaction_on_one(&self.constants.notes)?;
        let store = tx.object_store(&self.constants.notes)?;

        let commitment_proto = commitment.to_proto();

        let commitment_js = serde_wasm_bindgen::to_value(&commitment_proto)?;

        Ok(store
            .get_owned(commitment_js)?
            .await?
            .map(serde_wasm_bindgen::from_value)
            .transpose()?)
    }

    pub async fn get_chain_parameters(&self) -> WasmResult<Option<ChainParameters>> {
        let tx = self
            .db
            .transaction_on_one(&self.constants.chain_parameters)?;
        let store = tx.object_store(&self.constants.chain_parameters)?;

        Ok(store
            .get_owned("chain_parameters")?
            .await?
            .map(serde_wasm_bindgen::from_value)
            .transpose()?)
    }

    pub async fn get_fmd_parameters(&self) -> WasmResult<Option<FmdParameters>> {
        let tx = self.db.transaction_on_one(&self.constants.fmd_parameters)?;
        let store = tx.object_store(&self.constants.fmd_parameters)?;

        Ok(store
            .get_owned("fmd")?
            .await?
            .map(serde_wasm_bindgen::from_value)
            .transpose()?)
    }

    pub async fn get_swap_by_commitment(
        &self,
        swap_commitment: StateCommitment,
    ) -> WasmResult<Option<SwapRecord>> {
        let tx = self.db.transaction_on_one(&self.constants.swaps)?;
        let store = tx.object_store(&self.constants.swaps)?;

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
