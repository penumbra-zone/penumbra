use penumbra_crypto::dex::TradingPair;
use penumbra_crypto::Value;

/// Queries the chain for a transaction by hash.
#[derive(Debug, clap::Subcommand)]
pub enum ApproximateCmd {
    ConstantProduct {
        pair: TradingPair,
        quantity: Value,
    },
    Linear {
        pair: TradingPair,
        quantity_1: Value,
        quantity_2: Value,
    },
}

impl ApproximateCmd {
    pub fn offline(&self) -> bool {
        false
    }
}
