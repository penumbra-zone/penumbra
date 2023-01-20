use penumbra_crypto::dex::TradingPair;
use std::string::String;

pub fn position_nonce(nonce: &str) -> String {
    format!("dex/position_nonce/{}", nonce)
}

pub fn positions(trading_pair: &TradingPair, position_id: &str) -> String {
    format!("dex/positions/{}/opened/{}", trading_pair, position_id)
}

/// Encompasses non-consensus state keys.
pub(crate) mod internal {
    use penumbra_crypto::dex::lp::BareTradingFunction;

    pub fn prices(btf: &BareTradingFunction) -> [u8; 43] {
        let mut result: [u8; 43] = [0; 43];
        result[0..11].copy_from_slice("dex/prices/".as_bytes());
        result[11..43].copy_from_slice(&btf.to_bytes());
        result
    }
}
