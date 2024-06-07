use anyhow::Result;
use cometindex::Indexer;

pub mod shielded_pool;

#[tokio::main]
async fn main() -> Result<()> {
    Indexer::new()
        .with_default_tracing()
        .with_index(shielded_pool::fmd::ClueSet {})
        .run()
        .await?;

    Ok(())
}
