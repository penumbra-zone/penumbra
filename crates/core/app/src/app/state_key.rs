pub fn app_state() -> &'static str {
    "genesis/app_state"
}

pub fn transactions_by_height(block_height: u64) -> String {
    format!("cometbft-data/transactions_by_height/{block_height:020}/")
}

pub fn transaction_by_height_and_id(block_height: u64, tx_id: penumbra_transaction::Id) -> String {
    let tx_id = hex::encode(tx_id);
    format!("cometbft-data/transactions_by_height/{block_height:020}/{tx_id}")
}
