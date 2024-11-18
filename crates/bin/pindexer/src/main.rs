use anyhow::Result;
use clap::Parser as _;
use pindexer::{Indexer, IndexerExt as _, Options};

#[tokio::main]
async fn main() -> Result<()> {
    Indexer::new(Options::parse())
        .with_default_tracing()
        .with_default_penumbra_app_views()
        .run()
        .await?;

    Ok(())
}
