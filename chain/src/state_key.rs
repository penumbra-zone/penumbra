pub fn chain_params() -> &'static str {
    "chain_params"
}

pub fn block_height() -> &'static str {
    "block_height"
}

pub fn block_timestamp() -> &'static str {
    "block_timestamp"
}

pub fn fmd_parameters_current() -> &'static str {
    "fmd_parameters/current"
}

pub fn fmd_parameters_previous() -> &'static str {
    "fmd_parameters/previous"
}

// These are used for the object store:

pub fn halt_now() -> &'static str {
    "halt_now"
}

pub fn chain_params_changed() -> &'static str {
    "chain_params_changed"
}

pub fn epoch_by_height(height: u64) -> String {
    format!("chain/epoch_by_height/{}", height)
}
