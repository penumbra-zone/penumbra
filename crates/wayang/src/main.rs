use anyhow::Result;
use clap::Parser;
use penumbra_sdk_wayang::{config::Config, rhythm_and_feeler, Move};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::fs;
use tokio::task::JoinHandle;

async fn read_config(path: &Path) -> anyhow::Result<Config> {
    let data = fs::read_to_string(path).await?;
    Config::from_str(&data)
}

async fn write_example_config(path: &Path) -> anyhow::Result<()> {
    fs::write(path, Config::EXAMPLE_STR.as_bytes()).await?;
    Ok(())
}

#[derive(Parser)]
struct Args {
    /// Path to the TOML configuration file
    #[clap(short, long)]
    config: PathBuf,
}

impl Args {
    /// Read the config from the path provided in the arguments, or create a default one.
    async fn fetch_config(&self) -> anyhow::Result<Config> {
        let path = &self.config;
        if !fs::try_exists(path).await? {
            write_example_config(path).await?;
        }
        Ok(read_config(path).await?)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = args.fetch_config().await?;
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
