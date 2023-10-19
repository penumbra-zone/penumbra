use std::string::String;

pub fn chain_params() -> &'static str {
    "chain/params"
}

pub fn block_height() -> &'static str {
    "chain/block_height"
}

pub fn block_timestamp() -> &'static str {
    "chain/block_timestamp"
}

pub fn fmd_parameters_current() -> &'static str {
    "chain/fmd_parameters/current"
}

pub fn fmd_parameters_previous() -> &'static str {
    "chain/fmd_parameters/previous"
}

pub fn chain_halt_count() -> &'static str {
    "chain/halt_count"
}

// These are used in the nonverifiable store:
pub fn halted(total_halt_count: u64) -> Vec<u8> {
    let mut key = b"chain/halt/".to_vec();
    key.extend(total_halt_count.to_be_bytes());
    key
}

pub fn next_upgrade() -> &'static str {
    "chain/next_upgrade"
}

// These are used for the object store:
pub fn epoch_by_height(height: u64) -> String {
    format!("chain/epoch_by_height/{}", height)
}

pub fn epoch_change_at_height(height: u64) -> String {
    format!("chain/pending_epoch_changes/{}", height)
}

pub fn end_epoch_early() -> &'static str {
    "chain/end_epoch_early"
}

pub fn chain_params_updated() -> &'static str {
    "chain/chain_params_updated"
}
