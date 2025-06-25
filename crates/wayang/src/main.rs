use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Result;
use clap::Parser;
use penumbra_sdk_wayang::{config::Config, rhythm_and_feeler, Move};
use tokio::{fs, task::JoinHandle};

#[derive(Parser)]
struct Args {
    /// Path to the TOML configuration file
    #[clap(short, long)]
    config: PathBuf,
}

async fn read_config(path: &Path) -> anyhow::Result<Config> {
    let data = fs::read_to_string(path).await?;
    Config::from_str(&data)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = read_config(&args.config).await?;
    let (mut rhythm, feeler) = rhythm_and_feeler(&config)?;
    let rhythm_task: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        loop {
            let status = rhythm.sense().await?;
            dbg!(&status);
            rhythm
                .do_move(Move {
                    price: status.map(|x| x.price).unwrap_or_default() + 0.0001,
                })
                .await?;
        }
    });
    let feeler_task = tokio::spawn(feeler.run());
    tokio::select! {
        r = rhythm_task => r??,
        f = feeler_task => f??,
    }
    Ok(())
}
