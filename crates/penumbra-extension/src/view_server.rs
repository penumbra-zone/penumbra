use std::convert::TryInto;
use std::{collections::BTreeMap, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Error;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use penumbra_asset::asset::{DenomMetadata, Id};
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
    pub height: u64,
    pub sct_updates: Updates,
    pub new_notes: Vec<SpendableNoteRecord>,
    pub new_swaps: Vec<SwapRecord>,
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
