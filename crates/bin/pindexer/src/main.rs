use anyhow::Result;
use pindexer::{shielded_pool::fmd::ClueSet, Indexer};

#[tokio::main]
async fn main() -> Result<()> {
    Indexer::new()
        .with_default_tracing()
        .with_index(ClueSet {})
        .run()
        .await?;

    Ok(())
}
