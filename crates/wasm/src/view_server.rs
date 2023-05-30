use anyhow::Context;
use penumbra_compact_block::{CompactBlock, StatePayload};
use penumbra_crypto::asset::{DenomMetadata, Id};
use penumbra_crypto::{note, Asset, FullViewingKey, Nullifier};
use penumbra_dex::lp::position::Position;
use penumbra_dex::lp::LpNft;
use penumbra_proto::core::transaction::v1alpha1::{TransactionPerspective, TransactionView};
use penumbra_tct as tct;
use penumbra_tct::Witness::*;
use penumbra_transaction::Transaction;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::convert::TryInto;
use std::{collections::BTreeMap, str::FromStr};
use tct::storage::{StoreCommitment, StoreHash, StoredPosition, Updates};
use tct::{Forgotten, Tree};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use web_sys::console as web_console;

use crate::note_record::SpendableNoteRecord;
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
    nct_updates: Updates,
    new_notes: Vec<SpendableNoteRecord>,
    new_swaps: Vec<SwapRecord>,
}

impl ScanBlockResult {
    pub fn new(
        height: u64,
        nct_updates: Updates,
        new_notes: Vec<SpendableNoteRecord>,
        new_swaps: Vec<SwapRecord>,
    ) -> ScanBlockResult {
        Self {
            height,
            nct_updates,
            new_notes,
            new_swaps,
        }
    }
}

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

#[wasm_bindgen]
pub struct ViewServer {
    latest_height: u64,
    epoch_duration: u64,
    fvk: FullViewingKey,
    notes: BTreeMap<note::Commitment, SpendableNoteRecord>,
    notes_by_nullifier: BTreeMap<Nullifier, SpendableNoteRecord>,
    swaps: BTreeMap<tct::Commitment, SwapRecord>,
    denoms: BTreeMap<Id, DenomMetadata>,
    nct: penumbra_tct::Tree,
}

#[wasm_bindgen]
impl ViewServer {
    #[wasm_bindgen(constructor)]
    pub fn new(full_viewing_key: &str, epoch_duration: u64, stored_tree: JsValue) -> ViewServer {
        utils::set_panic_hook();
        let fvk = FullViewingKey::from_str(full_viewing_key.as_ref())
            .context("The provided string is not a valid FullViewingKey")
            .unwrap();

        let stored_tree: StoredTree = serde_wasm_bindgen::from_value(stored_tree).unwrap();

        let tree = load_tree(stored_tree);

        Self {
            latest_height: u64::MAX,
            fvk,
            epoch_duration,
            notes: Default::default(),
            notes_by_nullifier: Default::default(),
            denoms: Default::default(),
            nct: tree,
            swaps: Default::default(),
        }
    }

    #[wasm_bindgen]
    pub fn scan_block(
        &mut self,
        compact_block: JsValue,
        last_position: JsValue,
        last_forgotten: JsValue,
    ) -> JsValue {
        utils::set_panic_hook();

        let stored_position: Option<StoredPosition> =
            serde_wasm_bindgen::from_value(last_position).unwrap();
        let stored_forgotten: Option<Forgotten> =
            serde_wasm_bindgen::from_value(last_forgotten).unwrap();

        let block_proto: penumbra_proto::core::chain::v1alpha1::CompactBlock =
            serde_wasm_bindgen::from_value(compact_block).unwrap();

        let block: CompactBlock = block_proto.try_into().unwrap();

        let mut new_notes = Vec::new();
        let mut new_swaps: Vec<SwapRecord> = Vec::new();

        for state_payload in block.state_payloads {
            let clone_payload = state_payload.clone();

            match state_payload {
                StatePayload::Note { note: payload, .. } => {
                    match payload.trial_decrypt(&self.fvk) {
                        Some(note) => {
                            let note_position =
                                self.nct.insert(Keep, payload.note_commitment).unwrap();

                            let source = clone_payload.source().cloned().unwrap_or_default();
                            let nullifier = self
                                .fvk
                                .derive_nullifier(note_position, clone_payload.commitment());
                            let address_index = self
                                .fvk
                                .incoming()
                                .index_for_diversifier(note.diversifier());

                            web_console::log_1(&"Found new notes".into());

                            let note_record = SpendableNoteRecord {
                                note_commitment: clone_payload.commitment().clone(),
                                height_spent: None,
                                height_created: block.height,
                                note: note.clone(),
                                address_index,
                                nullifier,
                                position: note_position,
                                source,
                            };
                            new_notes.push(note_record.clone());
                            self.notes
                                .insert(payload.note_commitment, note_record.clone());
                            self.notes_by_nullifier
                                .insert(nullifier, note_record.clone());
                        }
                        None => {
                            self.nct.insert(Forget, payload.note_commitment).unwrap();
                        }
                    }
                }
                StatePayload::Swap { swap: payload, .. } => {
                    match payload.trial_decrypt(&self.fvk) {
                        Some(swap) => {
                            let swap_position = self.nct.insert(Keep, payload.commitment).unwrap();

                            let batch_data = block
                                .swap_outputs
                                .get(&swap.trading_pair)
                                .ok_or_else(|| anyhow::anyhow!("server gave invalid compact block"))
                                .unwrap();

                            let source = clone_payload.source().cloned().unwrap_or_default();
                            let nullifier = self
                                .fvk
                                .derive_nullifier(swap_position, clone_payload.commitment());

                            let swap_record = SwapRecord {
                                swap_commitment: clone_payload.commitment().clone(),
                                swap: swap.clone(),
                                position: swap_position,
                                nullifier,
                                source,
                                output_data: batch_data.clone(),
                                height_claimed: None,
                            };
                            new_swaps.push(swap_record.clone());
                            self.swaps.insert(payload.commitment, swap_record);
                        }
                        None => {
                            self.nct.insert(Forget, payload.commitment).unwrap();
                        }
                    }
                }
                StatePayload::RolledUp(commitment) => {
                    if self.notes.contains_key(&commitment) {
                        // This is a note we anticipated, so retain its auth path.
                        self.nct.insert(Keep, commitment).unwrap();
                    } else {
                        // This is someone else's note.
                        self.nct.insert(Forget, commitment).unwrap();
                    }
                }
            }
        }

        self.nct.end_block().unwrap();
        if block.epoch_root.is_some() {
            self.nct.end_epoch().unwrap();
        }

        self.latest_height = block.height;

        let nct_updates: Updates = self
            .nct
            .updates(
                stored_position.unwrap_or_default(),
                stored_forgotten.unwrap_or_default(),
            )
            .collect::<Updates>();

        let result = ScanBlockResult {
            height: self.latest_height,
            nct_updates,
            new_notes,
            new_swaps,
        };

        return serde_wasm_bindgen::to_value(&result).unwrap();
    }

    #[wasm_bindgen]
    pub fn scan_block_without_updates(&mut self, compact_block: JsValue) -> JsValue {
        utils::set_panic_hook();

        let block_proto: penumbra_proto::core::chain::v1alpha1::CompactBlock =
            serde_wasm_bindgen::from_value(compact_block).unwrap();

        let block: CompactBlock = block_proto.try_into().unwrap();

        // Newly detected spendable notes.
        let mut new_notes = Vec::new();
        // Newly detected claimable swaps.
        let mut new_swaps: Vec<SwapRecord> = Vec::new();

        for state_payload in block.state_payloads {
            let clone_payload = state_payload.clone();

            match state_payload {
                StatePayload::Note { note: payload, .. } => {
                    match payload.trial_decrypt(&self.fvk) {
                        Some(note) => {
                            let note_position =
                                self.nct.insert(Keep, payload.note_commitment).unwrap();

                            let source = clone_payload.source().cloned().unwrap_or_default();
                            let nullifier = self
                                .fvk
                                .derive_nullifier(note_position, clone_payload.commitment());
                            let address_index = self
                                .fvk
                                .incoming()
                                .index_for_diversifier(note.diversifier());

                            web_console::log_1(&"Found new notes".into());

                            let note_record = SpendableNoteRecord {
                                note_commitment: clone_payload.commitment().clone(),
                                height_spent: None,
                                height_created: block.height,
                                note: note.clone(),
                                address_index,
                                nullifier,
                                position: note_position,
                                source,
                            };
                            new_notes.push(note_record.clone());
                            self.notes
                                .insert(payload.note_commitment, note_record.clone());
                            self.notes_by_nullifier
                                .insert(nullifier, note_record.clone());
                        }
                        None => {
                            self.nct.insert(Forget, payload.note_commitment).unwrap();
                        }
                    }
                }
                StatePayload::Swap { swap: payload, .. } => {
                    match payload.trial_decrypt(&self.fvk) {
                        Some(swap) => {
                            let swap_position = self.nct.insert(Keep, payload.commitment).unwrap();
                            let batch_data = block
                                .swap_outputs
                                .get(&swap.trading_pair)
                                .ok_or_else(|| anyhow::anyhow!("server gave invalid compact block"))
                                .unwrap();

                            let source = clone_payload.source().cloned().unwrap_or_default();
                            let nullifier = self
                                .fvk
                                .derive_nullifier(swap_position, clone_payload.commitment());

                            let swap_record = SwapRecord {
                                swap_commitment: clone_payload.commitment().clone(),
                                swap: swap.clone(),
                                position: swap_position,
                                nullifier,
                                source,
                                output_data: batch_data.clone(),
                                height_claimed: None,
                            };
                            new_swaps.push(swap_record.clone());
                            self.swaps.insert(payload.commitment, swap_record);
                        }
                        None => {
                            self.nct.insert(Forget, payload.commitment).unwrap();
                        }
                    }
                }
                StatePayload::RolledUp(commitment) => {
                    if self.notes.contains_key(&commitment) {
                        // This is a note we anticipated, so retain its auth path.
                        self.nct.insert(Keep, commitment).unwrap();
                    } else {
                        // This is someone else's note.
                        self.nct.insert(Forget, commitment).unwrap();
                    }
                }
            }
        }

        self.nct.end_block().unwrap();
        if block.epoch_root.is_some() {
            self.nct.end_epoch().unwrap();
        }

        self.latest_height = block.height;

        let result = ScanBlockResult {
            height: self.latest_height,
            nct_updates: Default::default(),
            new_notes,
            new_swaps,
        };

        return serde_wasm_bindgen::to_value(&result).unwrap();
    }

    pub fn get_updates(&mut self, last_position: JsValue, last_forgotten: JsValue) -> JsValue {
        let stored_position: Option<StoredPosition> =
            serde_wasm_bindgen::from_value(last_position).unwrap();
        let stored_forgotten: Option<Forgotten> =
            serde_wasm_bindgen::from_value(last_forgotten).unwrap();

        let nct_updates: Updates = self
            .nct
            .updates(
                stored_position.unwrap_or_default(),
                stored_forgotten.unwrap_or_default(),
            )
            .collect::<Updates>();

        let result = ScanBlockResult {
            height: self.latest_height,
            nct_updates,
            new_notes: self.notes.clone().into_values().collect(),
            new_swaps: self.swaps.clone().into_values().collect(),
        };
        return serde_wasm_bindgen::to_value(&result).unwrap();
    }

    pub fn get_nct_root(&mut self) -> JsValue {
        let root = self.nct.root();
        return serde_wasm_bindgen::to_value(&root).unwrap();
    }

    pub fn transaction_info(&mut self, tx: JsValue) -> JsValue {
        let transaction = serde_wasm_bindgen::from_value(tx).unwrap();
        let (txp, txv) = self.transaction_info_inner(transaction);

        let txp_proto = TransactionPerspective::try_from(txp).unwrap();
        let txv_proto = TransactionView::try_from(txv).unwrap();

        let response = TxInfoResponse {
            txp: txp_proto,
            txv: txv_proto,
        };
        serde_wasm_bindgen::to_value(&response).unwrap()
    }

    pub fn get_lpnft_asset(
        &mut self,
        position_value: JsValue,
        position_state_value: JsValue,
    ) -> JsValue {
        let position: Position = serde_wasm_bindgen::from_value(position_value).unwrap();
        let position_state = serde_wasm_bindgen::from_value(position_state_value).unwrap();

        let lp_nft = LpNft::new(position.id(), position_state);
        let id = lp_nft.asset_id();
        let denom = lp_nft.denom();
        let asset = Asset { id, denom };

        serde_wasm_bindgen::to_value(&asset).unwrap()
    }
}

impl ViewServer {
    pub fn transaction_info_inner(
        &self,
        tx: Transaction,
    ) -> (
        penumbra_transaction::TransactionPerspective,
        penumbra_transaction::TransactionView,
    ) {
        // First, create a TxP with the payload keys visible to our FVK and no other data.

        let mut txp = penumbra_transaction::TransactionPerspective {
            payload_keys: tx
                .payload_keys(&self.fvk)
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
                    if let Some(spendable_note_record) = self.notes_by_nullifier.get(&nullifier) {
                        txp.spend_nullifiers
                            .insert(nullifier, spendable_note_record.note.clone());
                    }
                }
                Action::SwapClaim(claim) => {
                    let output_1_record = self
                        .notes
                        .get(&claim.body.output_1_commitment)
                        .expect("Error generating TxP: SwapClaim output 1 commitment not found");

                    let output_2_record = self
                        .notes
                        .get(&claim.body.output_2_commitment)
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
                    address_views.insert(address, self.fvk.view_address(address));
                    asset_ids.insert(note.asset_id());
                }
                ActionView::Output(OutputView::Visible { note, .. }) => {
                    let address = note.address();
                    address_views.insert(address, self.fvk.view_address(address));
                    asset_ids.insert(note.asset_id());
                }
                ActionView::Swap(SwapView::Visible { swap_plaintext, .. }) => {
                    let address = swap_plaintext.claim_address;
                    address_views.insert(address, self.fvk.view_address(address));
                    asset_ids.insert(swap_plaintext.trading_pair.asset_1());
                    asset_ids.insert(swap_plaintext.trading_pair.asset_2());
                }
                ActionView::SwapClaim(SwapClaimView::Visible {
                    output_1, output_2, ..
                }) => {
                    // Both will be sent to the same address so this only needs to be added once
                    let address = output_1.address();
                    address_views.insert(address, self.fvk.view_address(address));
                    asset_ids.insert(output_1.asset_id());
                    asset_ids.insert(output_2.asset_id());
                }
                ActionView::DelegatorVote(DelegatorVoteView::Visible { note, .. }) => {
                    let address = note.address();
                    address_views.insert(address, self.fvk.view_address(address));
                    asset_ids.insert(note.asset_id());
                }
                _ => {}
            }
        }

        // Now, extend the TxP with information helpful to understand the data it can view:

        let mut denoms = Vec::new();

        for id in asset_ids {
            if let Some(denom) = self.denoms.get(&id) {
                denoms.push(denom.clone());
            }
        }

        txp.denoms.extend(denoms.into_iter());

        txp.address_views = address_views.into_values().collect();

        // Finally, compute the full TxV from the full TxP:
        let txv = tx.view_from_perspective(&txp);

        (txp, txv)
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
    let tree = add_hashes.finish();
    return tree;
}
