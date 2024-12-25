pub mod config {
    pub fn sct_params() -> &'static str {
        "sct/config/sct_params"
    }
}

pub mod block_manager {
    pub fn block_height() -> &'static str {
        "sct/block_manager/block_height"
    }

    pub fn current_block_timestamp() -> &'static str {
        "sct/block_manager/block_timestamp"
    }

    pub fn block_timestamp(height: u64) -> String {
        format!("sct/block_manager/historical_block_timestamp/{}", height)
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

pub mod nullifier_set {
    use crate::Nullifier;

    pub fn spent_nullifier_lookup(nullifier: &Nullifier) -> String {
        format!("sct/nullifier_set/spent_nullifier_lookup/{}", nullifier)
    }

    pub fn pending_nullifiers() -> &'static str {
        "sct/nullifier_set/pending_nullifiers"
    }
}

pub mod tree {
    pub fn anchor_by_height(height: u64) -> String {
        format!("sct/tree/anchor_by_height/{}", height)
    }

    pub fn anchor_lookup(anchor: penumbra_sdk_tct::Root) -> String {
        format!("sct/tree/anchor_lookup/{}", anchor)
    }

    pub fn state_commitment_tree() -> &'static str {
        "sct/tree/state_commitment_tree"
    }

    pub fn note_source(note_commitment: &penumbra_sdk_tct::StateCommitment) -> String {
        format!("sct/tree/note_source/{}", note_commitment)
    }
}

pub mod cache {
    pub fn cached_state_commitment_tree() -> &'static str {
        "sct/cache/cached_state_commitment_tree"
    }
}

pub mod ambient {
    pub fn current_source() -> &'static str {
        "sct/ambient/current_source"
    }
}
