use anyhow::Result;
use pindexer::{Indexer, IndexerExt as _};

#[tokio::main]
async fn main() -> Result<()> {
    Indexer::new()
        .with_default_tracing()
        .with_default_penumbra_app_views()
        .run()
        .await?;

    Ok(())
}
