pub use cometindex::{AppView, ContextualizedEvent, Indexer, PgPool, PgTransaction};

mod indexer_ext;
pub use indexer_ext::IndexerExt;
use penumbra_asset::asset;
pub mod block;
pub mod dex_ex;
pub mod ibc;
pub mod insights;
mod parsing;
pub mod stake;
pub mod supply;

pub mod governance;

#[derive(clap::Parser, Clone, Debug)]
pub struct Options {
    #[clap(flatten)]
    pub cometindex: cometindex::opt::Options,
    /// The denom to use for indexing related components, of the form passet1...
    #[clap(
        long,
        default_value = "passet1w6e7fvgxsy6ccy3m8q0eqcuyw6mh3yzqu3uq9h58nu8m8mku359spvulf6"
    )]
    pub indexing_denom: asset::Id,
    /// The minimum liquidity for the indexing denom in the dex explorer app view.
    #[clap(long, default_value = "100000000")]
    pub dex_ex_min_liquidity: u128,
}
