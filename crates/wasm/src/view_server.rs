use std::convert::TryInto;
use std::{collections::BTreeMap, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Error;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use penumbra_asset::asset::{Id, Metadata};
use penumbra_compact_block::{CompactBlock, StatePayload};
use penumbra_dex::lp::position::Position;
use penumbra_dex::lp::LpNft;
use penumbra_keys::FullViewingKey;
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::note;
use penumbra_tct as tct;
use penumbra_tct::Witness::*;
use tct::storage::{StoreCommitment, StoreHash, StoredPosition, Updates};
use tct::{Forgotten, Tree};

use crate::error::WasmResult;
use crate::note_record::SpendableNoteRecord;
use crate::storage::IndexedDBStorage;
use crate::swap_record::SwapRecord;
use crate::utils;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredTree {
    last_position: Option<StoredPosition>,
    last_forgotten: Option<Forgotten>,
    hashes: Vec<StoreHash>,
    commitments: Vec<StoreCommitment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanBlockResult {
    height: u64,
    sct_updates: Updates,
    new_notes: Vec<SpendableNoteRecord>,
    new_swaps: Vec<SwapRecord>,
}

impl ScanBlockResult {
    pub fn new(
        height: u64,
        sct_updates: Updates,
        new_notes: Vec<SpendableNoteRecord>,
        new_swaps: Vec<SwapRecord>,
    ) -> ScanBlockResult {
        Self {
            height,
            sct_updates,
            new_notes,
            new_swaps,
        }
    }
}

#[wasm_bindgen]
pub struct ViewServer {
    latest_height: u64,
    epoch_duration: u64,
    fvk: FullViewingKey,
    notes: BTreeMap<note::StateCommitment, SpendableNoteRecord>,
    swaps: BTreeMap<tct::StateCommitment, SwapRecord>,
    denoms: BTreeMap<Id, Metadata>,
    sct: Tree,
    storage: IndexedDBStorage,
    last_position: Option<StoredPosition>,
    last_forgotten: Option<Forgotten>,
}

#[wasm_bindgen]
impl ViewServer {
    /// Create new instances of `ViewServer`
    /// Function opens a connection to indexedDb
    /// Arguments:
    ///     full_viewing_key: `bech32 string`
    ///     epoch_duration: `u64`
    ///     stored_tree: `StoredTree`
    ///     idb_constants: `IndexedDbConstants`
    /// Returns: `ViewServer`
    #[wasm_bindgen]
    pub async fn new(
        full_viewing_key: &str,
        epoch_duration: u64,
        stored_tree: JsValue,
        idb_constants: JsValue,
    ) -> WasmResult<ViewServer> {
        utils::set_panic_hook();

        let fvk = FullViewingKey::from_str(full_viewing_key)?;
        let stored_tree: StoredTree = serde_wasm_bindgen::from_value(stored_tree)?;
        let tree = load_tree(stored_tree);
        let constants = serde_wasm_bindgen::from_value(idb_constants)?;
        let view_server = Self {
            latest_height: u64::MAX,
            fvk,
            epoch_duration,
            notes: Default::default(),
            denoms: Default::default(),
            sct: tree,
            swaps: Default::default(),
            storage: IndexedDBStorage::new(constants).await?,
            last_position: None,
            last_forgotten: None,
        };
        Ok(view_server)
    }

    /// Scans block for notes, swaps
    /// Returns true if the block contains new notes, swaps or false if the block is empty for us
    ///     compact_block: `v1::CompactBlock`
    /// Scan results are saved in-memory rather than returned
    /// Use `flush_updates()` to get the scan results
    /// Returns: `bool`
    #[wasm_bindgen]
    pub async fn scan_block(&mut self, compact_block: JsValue) -> WasmResult<bool> {
        utils::set_panic_hook();

        let block_proto: penumbra_proto::core::component::compact_block::v1::CompactBlock =
            serde_wasm_bindgen::from_value(compact_block)?;

        let block: CompactBlock = block_proto.try_into()?;

        let mut found_new_data: bool = false;

        for state_payload in block.state_payloads {
            let clone_payload = state_payload.clone();

            match state_payload {
                StatePayload::Note { note: payload, .. } => {
                    match payload.trial_decrypt(&self.fvk) {
                        Some(note) => {
                            let note_position = self.sct.insert(Keep, payload.note_commitment)?;

                            let source = clone_payload.source().clone();
                            let nullifier = Nullifier::derive(
                                self.fvk.nullifier_key(),
                                note_position,
                                clone_payload.commitment(),
                            );
                            let address_index = self
                                .fvk
                                .incoming()
                                .index_for_diversifier(note.diversifier());

                            let note_record = SpendableNoteRecord {
                                note_commitment: *clone_payload.commitment(),
                                height_spent: None,
                                height_created: block.height,
                                note: note.clone(),
                                address_index,
                                nullifier,
                                position: note_position,
                                source,
                                return_address: None,
                            };
                            self.notes
                                .insert(payload.note_commitment, note_record.clone());
                            found_new_data = true;
                        }
                        None => {
                            self.sct.insert(Forget, payload.note_commitment)?;
                        }
                    }
                }
                StatePayload::Swap { swap: payload, .. } => {
                    match payload.trial_decrypt(&self.fvk) {
                        Some(swap) => {
                            let swap_position = self.sct.insert(Keep, payload.commitment)?;
                            let batch_data =
                                block.swap_outputs.get(&swap.trading_pair).ok_or_else(|| {
                                    anyhow::anyhow!("server gave invalid compact block")
                                })?;

                            let source = clone_payload.source().clone();
                            let nullifier = Nullifier::derive(
                                self.fvk.nullifier_key(),
                                swap_position,
                                clone_payload.commitment(),
                            );

                            let swap_record = SwapRecord {
                                swap_commitment: *clone_payload.commitment(),
                                swap: swap.clone(),
                                position: swap_position,
                                nullifier,
                                source,
                                output_data: *batch_data,
                                height_claimed: None,
                            };
                            self.swaps.insert(payload.commitment, swap_record);

                            let batch_data =
                                block.swap_outputs.get(&swap.trading_pair).ok_or_else(|| {
                                    anyhow::anyhow!("server gave invalid compact block")
                                })?;

                            let (output_1, output_2) = swap.output_notes(batch_data);

                            self.storage.store_advice(output_1).await?;
                            self.storage.store_advice(output_2).await?;
                            found_new_data = true;
                        }
                        None => {
                            self.sct.insert(Forget, payload.commitment)?;
                        }
                    }
                }
                StatePayload::RolledUp { commitment, .. } => {
                    // This is a note we anticipated, so retain its auth path.

                    let advice_result = self.storage.read_advice(commitment).await?;

                    match advice_result {
                        None => {
                            self.sct.insert(Forget, commitment)?;
                        }
                        Some(note) => {
                            let position = self.sct.insert(Keep, commitment)?;

                            let address_index_1 = self
                                .fvk
                                .incoming()
                                .index_for_diversifier(note.diversifier());

                            let nullifier =
                                Nullifier::derive(self.fvk.nullifier_key(), position, &commitment);

                            let source = clone_payload.source().clone();

                            let spendable_note = SpendableNoteRecord {
                                note_commitment: note.commit(),
                                height_spent: None,
                                height_created: block.height,
                                note: note.clone(),
                                address_index: address_index_1,
                                nullifier,
                                position,
                                source,
                                return_address: None,
                            };
                            self.notes
                                .insert(spendable_note.note_commitment, spendable_note.clone());
                            found_new_data = true;
                        }
                    }
                }
            }
        }

        self.sct.end_block()?;
        if block.epoch_root.is_some() {
            self.sct.end_epoch()?;
        }

        self.latest_height = block.height;

        Ok(found_new_data)
    }

    /// Get new notes, swaps, SCT state updates
    /// Function also clears state
    /// Returns: `ScanBlockResult`
    #[wasm_bindgen]
    pub fn flush_updates(&mut self) -> WasmResult<JsValue> {
        utils::set_panic_hook();

        let sct_updates: Updates = self
            .sct
            .updates(
                self.last_position.unwrap_or_default(),
                self.last_forgotten.unwrap_or_default(),
            )
            .collect::<Updates>();

        let updates = ScanBlockResult {
            height: self.latest_height,
            sct_updates: sct_updates.clone(),
            new_notes: self.notes.clone().into_values().collect(),
            new_swaps: self.swaps.clone().into_values().collect(),
        };

        self.notes = Default::default();
        self.swaps = Default::default();

        self.last_position = sct_updates.set_position;
        self.last_forgotten = sct_updates.set_forgotten;

        let serializer = Serializer::new().serialize_large_number_types_as_bigints(true);
        let result = updates.serialize(&serializer)?;
        Ok(result)
    }

    /// get SCT root
    /// SCT root can be compared with the root obtained by GRPC and verify that there is no divergence
    /// Returns: `Root`
    #[wasm_bindgen]
    pub fn get_sct_root(&mut self) -> Result<JsValue, Error> {
        utils::set_panic_hook();

        let root = self.sct.root();
        serde_wasm_bindgen::to_value(&root)
    }

    /// get LP NFT asset
    /// Arguments:
    ///     position_value: `lp::position::Position`
    ///     position_state_value: `lp::position::State`
    /// Returns: `DenomMetadata`
    #[wasm_bindgen]
    pub fn get_lpnft_asset(
        &mut self,
        position_value: JsValue,
        position_state_value: JsValue,
    ) -> Result<JsValue, Error> {
        utils::set_panic_hook();

        let position: Position = serde_wasm_bindgen::from_value(position_value)?;
        let position_state = serde_wasm_bindgen::from_value(position_state_value)?;
        let lp_nft = LpNft::new(position.id(), position_state);
        let denom = lp_nft.denom();
        serde_wasm_bindgen::to_value(&denom)
    }
}

pub fn load_tree(stored_tree: StoredTree) -> Tree {
    let stored_position: StoredPosition = stored_tree.last_position.unwrap_or_default();
    let mut add_commitments = Tree::load(
        stored_position,
        stored_tree.last_forgotten.unwrap_or_default(),
    );

    for store_commitment in &stored_tree.commitments {
        add_commitments.insert(store_commitment.position, store_commitment.commitment)
    }
    let mut add_hashes = add_commitments.load_hashes();

    for stored_hash in &stored_tree.hashes {
        add_hashes.insert(stored_hash.position, stored_hash.height, stored_hash.hash);
    }
    add_hashes.finish()
}
