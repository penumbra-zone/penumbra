use penumbra_crypto::{note, Nullifier};
use penumbra_tct::{
    builder::{block, epoch},
    Root,
};
use std::string::String;

pub fn anchor_by_height(height: u64) -> String {
    format!("shielded_pool/anchor/{height}")
}

pub fn anchor_lookup(anchor: Root) -> String {
    format!("shielded_pool/valid_anchors/{anchor}")
}

pub fn stub_state_commitment_tree() -> &'static str {
    "shielded_pool/stub/state_commitment_tree"
}

pub fn block_anchor_by_height(height: u64) -> String {
    format!("shielded_pool/block_anchor/{height}")
}

pub fn block_anchor_lookup(anchor: block::Root) -> String {
    format!("shielded_pool/valid_block_anchors/{anchor}")
}

pub fn note_source(note_commitment: &note::Commitment) -> String {
    format!("shielded_pool/note_source/{note_commitment}")
}

pub fn epoch_anchor_lookup(anchor: epoch::Root) -> String {
    format!("shielded_pool/valid_epoch_anchors/{anchor}")
}

pub fn spent_nullifier_lookup(nullifier: &Nullifier) -> String {
    format!("shielded_pool/spent_nullifiers/{nullifier}")
}

pub fn epoch_anchor_by_index(index: u64) -> String {
    format!("shielded_pool/epoch_anchor/{index}")
}
