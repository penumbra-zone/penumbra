use anyhow::Result;
use clap::Parser;

use pclientd::Opt;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::parse();

    opt.exec().await
}
