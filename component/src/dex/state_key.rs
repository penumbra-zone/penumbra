use penumbra_crypto::dex::{lp::position, DirectedTradingPair, TradingPair};
use std::string::String;

pub fn position_nonce(nonce: &[u8]) -> String {
    format!("dex/position_nonce/{:?}", nonce)
}

pub fn positions(trading_pair: &TradingPair, position_id: &str) -> String {
    format!("dex/positions/{}/opened/{}", trading_pair, position_id)
}

/// Looks up a `PositionMetadata` by its ID
pub fn position_by_id(id: &position::Id) -> String {
    format!("dex/position/{}", id)
}

/// Encompasses non-consensus state keys.
pub(crate) mod internal {
    use super::*;
    use penumbra_crypto::dex::lp::BareTradingFunction;

    pub mod price_index {
        use super::*;
        pub fn prefix(pair: &DirectedTradingPair) -> [u8; 71] {
            let mut key = [0u8; 71];
            key[0..7].copy_from_slice(b"dex/pi/");
            key[7..7 + 32].copy_from_slice(&pair.start.to_bytes());
            key[7 + 32..7 + 32 + 32].copy_from_slice(&pair.end.to_bytes());
            key
        }

        pub fn key(pair: &DirectedTradingPair, btf: &BareTradingFunction) -> Vec<u8> {
            let mut key = [0u8; 103];
            key[0..71].copy_from_slice(&prefix(pair));
            key[71..103].copy_from_slice(&btf.effective_price_key_bytes());
            key.to_vec()
        }
    }
}
