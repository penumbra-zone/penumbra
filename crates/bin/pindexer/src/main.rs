use anyhow::Result;
use clap::Parser as _;
use pindexer::{Indexer, IndexerExt as _, Options};

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Options::parse();
    let cometindex_opts = opts.cometindex.clone();
    match cometindex_opts.command {
        cometindex::opt::Command::Index(index_options) => {
            Indexer::new(cometindex_opts.src_database_url, index_options)
                .with_default_tracing()
                .with_default_penumbra_app_views(&opts)
                .run()
                .await?;
        }
        cometindex::opt::Command::IntegrityCheck => todo!(),
    }

    Ok(())
}
