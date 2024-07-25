use anyhow::Result;
use clap::Parser as _;
use pindexer::{block::Block, block_events::BlockEvents};
use pindexer::{Indexer, IndexerExt as _, Options};

#[tokio::main]
async fn main() -> Result<()> {
    Indexer::new(Options::parse())
        .with_default_tracing()
        .with_default_penumbra_app_views()
        .with_index(Block {})
        .with_index(BlockEvents {})
        .run()
        .await?;

    Ok(())
}
