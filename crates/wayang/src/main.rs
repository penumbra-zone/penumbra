use anyhow::Result;
use clap::Parser;
use penumbra_sdk_wayang::{config::Config, init_tracing, rhythm_and_feeler, Move, Position};
use std::{path::PathBuf, str::FromStr};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::task::JoinHandle;

#[derive(Parser)]
struct Args {
    /// Path to the TOML configuration file
    #[clap(short, long)]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;

    let args = Args::parse();
    let config = Config::fetch(&args.config).await?;
    let (mut rhythm, feeler) = rhythm_and_feeler(&config).await?;
    let rhythm_task: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            let status = rhythm.sense().await?;
            tracing::info!(?status, "current status");

            let position = loop {
                line.clear();
                reader.read_line(&mut line).await?;
                let trimmed = line.trim();
                if let Ok(position) = Position::from_str(trimmed) {
                    break position;
                } else {
                    tracing::info!("invalid position: '{trimmed}'");
                }
            };

            tracing::info!(%position, "submitting position move");
            rhythm.do_move(Move { position }).await?;
        }
    });
    let feeler_task = tokio::spawn(feeler.run());
    tokio::select! {
        r = rhythm_task => r??,
        f = feeler_task => f??,
    }
    Ok(())
}
