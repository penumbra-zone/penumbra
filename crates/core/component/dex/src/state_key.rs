use std::string::String;

use penumbra_asset::asset;

use crate::{lp::position, DirectedTradingPair, TradingPair};

pub mod config {
    pub fn dex_params() -> &'static str {
        "dex/config/dex_params"
    }

    pub fn dex_params_updated() -> &'static str {
        "dex/config/dex_params_updated"
    }
}

pub fn value_balance(asset_id: &asset::Id) -> String {
    format!("dex/value_balance/{asset_id}")
}

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

pub fn aggregate_value() -> &'static str {
    "dex/aggregate_value"
}

pub(crate) mod engine {
    use super::*;
    use crate::lp::BareTradingFunction;

    pub(crate) mod counter {
        pub(crate) mod num_positions {
            use crate::TradingPair;

            pub(crate) fn prefix() -> &'static str {
                "dex/internal/counter/num_positions/"
            }

            pub(crate) fn by_trading_pair(trading_pair: &TradingPair) -> [u8; 99] {
                let mut key = [0u8; 99];
                let prefix_bytes = prefix().as_bytes();
                let canonical_pair_bytes = trading_pair.to_bytes();

                key[0..35].copy_from_slice(prefix_bytes);
                key[35..99].copy_from_slice(&canonical_pair_bytes);
                key
            }
        }
    }

    /// Find assets with liquidity positions from asset `from`, ordered by price.
    pub(crate) mod routable_assets {
        use penumbra_asset::asset;
        use penumbra_num::Amount;

        use super::*;

        /// `A || be_bytes(A_from_B) => B` this will be an ordered encoding of every asset `B` directly routable to from `A`.
        /// `a_from_b` represents the amount of `A` that can be bought with `B`.
        /// The prefix query includes only the `A` portion, meaning the keys will be returned in order of liquidity.
        pub(crate) fn prefix(from: &asset::Id) -> [u8; 39] {
            let mut key = [0u8; 39];
            key[0..7].copy_from_slice(b"dex/ra/");
            key[7..7 + 32].copy_from_slice(&from.to_bytes());
            key
        }

        /// `A || be_bytes(A_from_B) => B` this will be an ordered encoding of every asset `B` directly routable to from `A`.
        /// `a_from_b` represents the amount of `A` that can be bought with `B`.
        pub(crate) fn key(from: &asset::Id, a_from_b: Amount) -> [u8; 55] {
            let mut key = [0u8; 55];
            key[0..7].copy_from_slice(b"dex/ra/");
            key[7..32 + 7].copy_from_slice(&from.to_bytes());
            key[32 + 7..32 + 7 + 16].copy_from_slice(&a_from_b.to_be_bytes());
            key
        }

        /// `(A, B) => A_from_B` this will encode the current amount of `A` tradable into `B` for every directly routable trading pair.
        /// This index can be used to determine the key values for the [`super::key`] ordered index to perform updates efficiently.
        pub(crate) fn a_from_b(pair: &DirectedTradingPair) -> [u8; 71] {
            let mut key = [0u8; 71];
            key[0..7].copy_from_slice(b"dex/ab/");
            key[7..7 + 32].copy_from_slice(&pair.start.to_bytes());
            key[7 + 32..7 + 32 + 32].copy_from_slice(&pair.end.to_bytes());
            key
        }
    }

    pub(crate) mod price_index {
        use super::*;

        pub(crate) fn prefix(pair: &DirectedTradingPair) -> [u8; 71] {
            let mut key = [0u8; 71];
            key[0..7].copy_from_slice(b"dex/pi/");
            key[7..7 + 32].copy_from_slice(&pair.start.to_bytes());
            key[7 + 32..7 + 32 + 32].copy_from_slice(&pair.end.to_bytes());
            key
        }

        pub(crate) fn key(
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

pub(crate) mod eviction_queue {
    pub mod inventory_index {
        use crate::lp::position;
        use crate::DirectedTradingPair;
        use penumbra_num::Amount;

        pub fn by_trading_pair(pair: &DirectedTradingPair) -> [u8; 107] {
            let mut prefix = [0u8; 107];
            prefix[0..43].copy_from_slice(b"dex/internal/eviction_queue/inventory_index");
            prefix[43..75].copy_from_slice(&pair.start.to_bytes());
            prefix[75..107].copy_from_slice(&pair.end.to_bytes());
            prefix
        }

        pub fn key(pair: &DirectedTradingPair, inventory: Amount, id: &position::Id) -> [u8; 155] {
            let mut full_key = [0u8; 155];
            let prefix = by_trading_pair(pair);
            full_key[0..107].copy_from_slice(&prefix);
            full_key[107..123].copy_from_slice(&inventory.to_be_bytes());
            full_key[123..155].copy_from_slice(&id.0);

            full_key
        }
    }
}

pub(crate) mod eviction_queue {
    pub(crate) mod inventory_index {
        use crate::lp::position;
        use crate::DirectedTradingPair;
        use penumbra_num::Amount;

        pub(crate) fn by_trading_pair(pair: &DirectedTradingPair) -> [u8; 107] {
            let mut prefix = [0u8; 107];
            prefix[0..43].copy_from_slice(b"dex/internal/eviction_queue/inventory_index");
            prefix[43..75].copy_from_slice(&pair.start.to_bytes());
            prefix[75..107].copy_from_slice(&pair.end.to_bytes());
            prefix
        }

        pub(crate) fn key(
            pair: &DirectedTradingPair,
            inventory: Amount,
            id: &position::Id,
        ) -> [u8; 155] {
            let mut full_key = [0u8; 155];
            let prefix = by_trading_pair(pair);
            full_key[0..107].copy_from_slice(&prefix);
            full_key[107..123].copy_from_slice(&inventory.to_be_bytes());
            full_key[123..155].copy_from_slice(&id.0);

            full_key
        }
    }
}
