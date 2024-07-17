use anyhow::Result;
use pindexer::{Indexer, IndexerExt as _};
use pindexer::block::block::Block;

#[tokio::main]
async fn main() -> Result<()> {
    dbg!("hello");
    Indexer::new()
        .with_default_tracing()
        .with_default_penumbra_app_views()
        .with_index(Block {})
        .run()
        .await?;

    Ok(())
}
