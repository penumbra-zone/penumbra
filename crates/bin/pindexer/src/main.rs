use anyhow::Result;
use pindexer::block::block::Block;
use pindexer::{Indexer, IndexerExt as _};

#[tokio::main]
async fn main() -> Result<()> {
    Indexer::new()
        .with_default_tracing()
        .with_default_penumbra_app_views()
        .with_index(Block {})
        .run()
        .await?;

    Ok(())
}
