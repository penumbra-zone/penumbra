pub mod genesis {
    pub fn app_state() -> &'static str {
        "application/genesis/app_state"
    }
}

pub mod cometbft_data {
    use crate::COMETBFT_SUBSTORE_PREFIX;

    pub fn transactions_by_height(block_height: u64) -> String {
        format!(
            "{}/transactions_by_height/{block_height:020}",
            COMETBFT_SUBSTORE_PREFIX
        )
    }
}

pub mod counters {
    pub fn halt_count() -> &'static str {
        "application/counters/halt_count"
    }
}
