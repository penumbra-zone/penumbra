use std::string::String;

use penumbra_tct::{Root, StateCommitment};

use crate::Nullifier;

pub fn sct_params() -> &'static str {
    "sct/params"
}

pub fn sct_params_updated() -> &'static str {
    "sct/sct_params_updated"
}

pub mod block_manager {
    pub fn block_height() -> &'static str {
        "sct/block_manager/block_height"
    }

    pub fn block_timestamp() -> &'static str {
        "sct/block_manager/block_timestamp"
    }
}

pub mod epoch_manager {
    pub fn epoch_by_height(height: u64) -> String {
        format!("sct/epoch_manager/epoch_by_height/{}", height)
    }

    pub fn epoch_change_at_height(height: u64) -> String {
        format!("sct/epoch_manager/pending_epoch_changes/{}", height)
    }

    pub fn end_epoch_early() -> &'static str {
        "sct/epoch_manager/end_epoch_early"
    }
}

pub fn spent_nullifier_lookup(nullifier: &Nullifier) -> String {
    format!("sct/nf/{nullifier}")
}

pub fn pending_nullifiers() -> &'static str {
    "sct/pending_nullifiers"
}

pub fn anchor_by_height(height: u64) -> String {
    format!("sct/anchor/{height}")
}

pub fn anchor_lookup(anchor: Root) -> String {
    format!("sct/valid_anchors/{anchor}")
}

pub fn state_commitment_tree() -> &'static str {
    "sct/state_commitment_tree"
}

pub fn note_source(note_commitment: &StateCommitment) -> String {
    format!("sct/note_source/{note_commitment}")
}

// In-memory state key for caching the current SCT (avoids serialization overhead)
pub fn cached_state_commitment_tree() -> &'static str {
    "sct/cached_state_commitment_tree"
}

pub fn current_source() -> &'static str {
    "sct/current_source"
}
