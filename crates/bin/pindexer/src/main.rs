use anyhow::Result;
use clap::Parser as _;
use pindexer::{supply::Supply, Indexer, Options};

#[tokio::main]
async fn main() -> Result<()> {
    Indexer::new(Options::parse())
        .with_default_tracing()
        .with_index(Supply::new())
        .run()
        .await?;

    Ok(())
}
