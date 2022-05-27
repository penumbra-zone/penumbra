use jmt::KeyHash;

pub fn chain_params() -> KeyHash {
    format!("chain_params").into()
}

pub fn block_height() -> KeyHash {
    format!("block_height").into()
}
