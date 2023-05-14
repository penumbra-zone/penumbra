use std::string::String;

use penumbra_tct::{
    builder::{block, epoch},
    Root,
};

pub fn anchor_by_height(height: u64) -> String {
    format!("sct/anchor/{height}")
}

pub fn anchor_lookup(anchor: Root) -> String {
    format!("sct/valid_anchors/{anchor}")
}

pub fn stub_state_commitment_tree() -> &'static str {
    "sct/stub/state_commitment_tree"
}

pub fn block_anchor_by_height(height: u64) -> String {
    format!("sct/block_anchor/{height}")
}

pub fn block_anchor_lookup(anchor: block::Root) -> String {
    format!("sct/valid_block_anchors/{anchor}")
}

pub fn epoch_anchor_lookup(anchor: epoch::Root) -> String {
    format!("sct/valid_epoch_anchors/{anchor}")
}

pub fn epoch_anchor_by_index(index: u64) -> String {
    format!("sct/epoch_anchor/{index}")
}
