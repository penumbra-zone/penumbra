use penumbra_sdk_sct::Nullifier;

pub fn staking_funding_parameters() -> &'static str {
    "funding/parameters"
}

pub fn lqt_nullifier_lookup_for_txid(epoch_index: u64, nullifier: &Nullifier) -> String {
    format!("funding/lqt/v1/nf/by_epoch/{epoch_index:020}/lookup/{nullifier}")
}
