use std::string::String;

use crate::{lp::position, DirectedTradingPair, TradingPair};

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

pub fn swap_execution(height: u64, trading_pair: DirectedTradingPair) -> String {
    format!(
        "dex/swap_execution/{:020}/{}/{}",
        height, &trading_pair.start, &trading_pair.end
    )
}

pub fn swap_executions() -> &'static str {
    "dex/swap_execution/"
}

pub fn arb_execution(height: u64) -> String {
    format!("dex/arb_execution/{height:020}")
}

pub fn arb_executions() -> &'static str {
    "dex/arb_execution/"
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
    use crate::lp::BareTradingFunction;

    /// Find assets with liquidity positions from asset `from`, ordered by price.
    pub mod routable_assets {
        use penumbra_asset::asset;
        use penumbra_num::Amount;

        use super::*;

        /// `A || be_bytes(A_from_B) => B` this will be an ordered encoding of every asset `B` directly routable to from `A`.
        /// `a_from_b` represents the amount of `A` that can be bought with `B`.
        /// The prefix query includes only the `A` portion, meaning the keys will be returned in order of liquidity.
        pub fn prefix(from: &asset::Id) -> [u8; 39] {
            let mut key = [0u8; 39];
            key[0..7].copy_from_slice(b"dex/ra/");
            key[7..7 + 32].copy_from_slice(&from.to_bytes());
            key
        }

        /// `A || be_bytes(A_from_B) => B` this will be an ordered encoding of every asset `B` directly routable to from `A`.
        /// `a_from_b` represents the amount of `A` that can be bought with `B`.
        pub fn key(from: &asset::Id, a_from_b: Amount) -> [u8; 55] {
            let mut key = [0u8; 55];
            key[0..7].copy_from_slice(b"dex/ra/");
            key[7..32 + 7].copy_from_slice(&from.to_bytes());
            key[32 + 7..32 + 7 + 16].copy_from_slice(&a_from_b.to_be_bytes());
            key
        }

        /// `(A, B) => A_from_B` this will encode the current amount of `A` tradable into `B` for every directly routable trading pair.
        /// This index can be used to determine the key values for the [`super::key`] ordered index to perform updates efficiently.
        pub fn a_from_b(pair: &TradingPair) -> [u8; 61] {
            let mut key = [0u8; 61];
            key[0..7].copy_from_slice(b"dex/ab/");
            key[7..7 + 32].copy_from_slice(&pair.asset_1.to_bytes());
            key[7 + 32..7 + 32 + 32].copy_from_slice(&pair.asset_2.to_bytes());
            key
        }
    }

    pub mod price_index {
        use super::*;

        pub fn prefix(pair: &DirectedTradingPair) -> [u8; 71] {
            let mut key = [0u8; 71];
            key[0..7].copy_from_slice(b"dex/pi/");
            key[7..7 + 32].copy_from_slice(&pair.start.to_bytes());
            key[7 + 32..7 + 32 + 32].copy_from_slice(&pair.end.to_bytes());
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
