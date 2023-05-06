use std::path::PathBuf;

use penumbra_crypto::dex::Market;
use penumbra_crypto::Value;

/// Queries the chain for a transaction by hash.
#[derive(Debug, clap::Subcommand)]
pub enum ApproximateCmd {
    #[clap(visible_alias = "xyk")]
    ConstantProduct {
        market: Market,
        quantity: Value,
        #[clap(short, long)]
        current_price: Option<f64>,
        #[clap(short, long, hide(true))]
        debug_file: Option<PathBuf>,
    },
}

impl ApproximateCmd {
    pub fn offline(&self) -> bool {
        false
    }
}
