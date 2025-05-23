use anyhow::Result;
use clap::Parser;
use penumbra_sdk_wayang::{options::Options, rhythm_and_feeler, Move};
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::parse();
    let (mut rhythm, feeler) = rhythm_and_feeler(options);
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
