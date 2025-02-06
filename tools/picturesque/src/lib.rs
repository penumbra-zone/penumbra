use clap::Parser;
use std::{path::PathBuf, time::Duration};

mod cometbft;
mod pd;
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
    let root = options.directory.canonicalize()?;
    let postgres = tokio::spawn(postgres::run(root.clone()));
    let cometbft = tokio::spawn(cometbft::run(
        root.clone(),
        Some(Duration::from_millis(4000)),
    ));
    let pd = tokio::spawn(pd::run(root.clone(), Some(Duration::from_millis(8000))));
    tokio::select! {
        x = postgres => x??,
        x = cometbft => x??,
        x = pd => x??,
    };
    Ok(())
}
