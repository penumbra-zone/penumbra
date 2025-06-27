use anyhow::Result;
use clap::Parser;
use penumbra_sdk_wayang::{config::Config, rhythm_and_feeler, Move, PositionShape};
use std::{
    io::IsTerminal,
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::fs;
use tokio::task::JoinHandle;
use tracing_subscriber::{prelude::*, EnvFilter};

#[tracing::instrument]
async fn read_config(path: &Path) -> anyhow::Result<Config> {
    let data = fs::read_to_string(path).await?;
    Config::from_str(&data)
}

#[tracing::instrument]
async fn write_example_config(path: &Path) -> anyhow::Result<()> {
    tracing::info!("creating example config file");
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
    #[tracing::instrument(skip(self))]
    async fn fetch_config(&self) -> anyhow::Result<Config> {
        let path = &self.config;
        if !fs::try_exists(path).await? {
            tracing::info!("Config file not found, creating default.");
            write_example_config(path).await?;
        }
        tracing::info!("Loading config.");
        Ok(read_config(path).await?)
    }
}

fn init_tracing() -> anyhow::Result<()> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(std::io::stdout().is_terminal())
        .with_writer(std::io::stderr)
        .with_target(true);
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);
    registry.init();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;

    let args = Args::parse();
    let config = args.fetch_config().await?;
    let (mut rhythm, feeler) = rhythm_and_feeler(&config)?;
    let rhythm_task: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        loop {
            let status = rhythm.sense().await?;
            let price = status
                .and_then(|x| {
                    x.positions
                        .get(0)
                        .map(|x| (x.shape.upper_price + x.shape.lower_price) / 2.0)
                })
                .unwrap_or_default()
                + 0.0001;
            let shape = PositionShape {
                upper_price: 1.01 * price,
                lower_price: 0.99 * price,
                base_liquidity: 1.0,
                quote_liquidity: 1.0,
            };
            tracing::info!(?shape, "submitting position move");
            rhythm.do_move(Move { shape }).await?;
        }
    });
    let feeler_task = tokio::spawn(feeler.run());
    tokio::select! {
        r = rhythm_task => r??,
        f = feeler_task => f??,
    }
    Ok(())
}
