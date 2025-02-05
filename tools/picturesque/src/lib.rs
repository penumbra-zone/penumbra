use clap::Parser;
use std::path::PathBuf;

mod postgres;

/// This struct represents the command-line options
#[derive(Clone, Debug, Parser)]
#[clap(name = "picturesque", about = "a simple tool to run devnets", version)]
pub struct Options {
    /// The base of operations for the tool.
    #[clap(short, long)]
    pub directory: PathBuf,
}

#[tracing::instrument]
pub async fn run_devnet(options: Options) -> anyhow::Result<()> {
    tracing::info!("spawning postgres");
    let postgres = tokio::spawn(postgres::run(options.directory.canonicalize()?));
    tokio::select! {
        x = postgres => x??
    };
    Ok(())
}
