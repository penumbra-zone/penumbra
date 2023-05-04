use penumbra_crypto::dex::Market;
use penumbra_crypto::Value;

/// Queries the chain for a transaction by hash.
#[derive(Debug, clap::Subcommand)]
pub enum ApproximateCmd {
    ConstantProduct { market: Market, quantity: Value },
}

impl ApproximateCmd {
    pub fn offline(&self) -> bool {
        false
    }
}
