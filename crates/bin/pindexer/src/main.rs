use anyhow::Result;
use clap::Parser as _;
use pindexer::{Indexer, IndexerExt as _, Options};

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Options::parse();
    Indexer::new(opts.cometindex.clone())
        .with_default_tracing()
        .with_default_penumbra_app_views(&opts)
        .run()
        .await?;

    Ok(())
}
