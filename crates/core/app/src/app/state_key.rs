pub fn app_state() -> &'static str {
    "genesis/app_state"
}

pub fn transactions_by_height(block_height: u64) -> String {
    format!("cometbft-data/transactions_by_height/{block_height:020}")
}
