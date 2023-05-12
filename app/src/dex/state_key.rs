use penumbra_crypto::dex::{lp::position, DirectedTradingPair, TradingPair};
use std::string::String;

pub fn positions(trading_pair: &TradingPair, position_id: &str) -> String {
    format!("dex/positions/{trading_pair}/opened/{position_id}")
}

/// Looks up a `Position` by its ID
pub fn position_by_id(id: &position::Id) -> String {
    format!("dex/position/{id}")
}

pub fn all_positions() -> &'static str {
    "dex/position/"
}

pub fn output_data(height: u64, trading_pair: TradingPair) -> String {
    format!(
        "dex/output/{:020}/{}/{}",
        height,
        &trading_pair.asset_1(),
        &trading_pair.asset_2()
    )
}

pub fn swap_execution(height: u64, trading_pair: TradingPair) -> String {
    format!(
        "dex/swap_execution/{:020}/{}/{}",
        height,
        &trading_pair.asset_1(),
        &trading_pair.asset_2()
    )
}

pub fn swap_flows() -> &'static str {
    "dex/swap_flows"
}

pub fn pending_position_closures() -> &'static str {
    "dex/pending_position_closures"
}

pub fn pending_payloads() -> &'static str {
    "dex/pending_payloads"
}

pub fn pending_outputs() -> &'static str {
    "dex/pending_outputs"
}

/// Encompasses non-consensus state keys.
pub(crate) mod internal {
    use super::*;
    use penumbra_crypto::dex::lp::BareTradingFunction;

    pub mod price_index {
        use penumbra_crypto::asset;

        use super::*;

        pub fn prefix(pair: &DirectedTradingPair) -> [u8; 71] {
            let mut key = [0u8; 71];
            key[0..7].copy_from_slice(b"dex/pi/");
            key[7..7 + 32].copy_from_slice(&pair.start.to_bytes());
            key[7 + 32..7 + 32 + 32].copy_from_slice(&pair.end.to_bytes());
            key
        }

        /// A prefix that will return all positions "from" a given asset.
        pub fn from_asset_prefix(from: &asset::Id) -> [u8; 39] {
            let mut key = [0u8; 39];
            key[0..7].copy_from_slice(b"dex/pi/");
            key[7..7 + 32].copy_from_slice(&from.to_bytes());
            key
        }

        pub fn key(
            pair: &DirectedTradingPair,
            btf: &BareTradingFunction,
            id: &position::Id,
        ) -> Vec<u8> {
            let id_bytes = id.0;
            let mut key = [0u8; 135];
            key[0..71].copy_from_slice(&prefix(pair));
            key[71..103].copy_from_slice(&btf.effective_price_key_bytes());
            key[103..135].copy_from_slice(&id_bytes);
            key.to_vec()
        }
    }
}
