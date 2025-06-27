use anyhow::Result;
use clap::Parser;
use penumbra_sdk_wayang::{config::Config, init_tracing, rhythm_and_feeler, Move, PositionShape};
use std::path::PathBuf;
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
