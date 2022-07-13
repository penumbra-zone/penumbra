use jmt::KeyHash;

pub fn chain_params() -> KeyHash {
    "chain_params".to_string().into()
}

pub fn block_height() -> KeyHash {
    "block_height".to_string().into()
}

pub fn block_timestamp() -> KeyHash {
    "block_timestamp".to_string().into()
}
