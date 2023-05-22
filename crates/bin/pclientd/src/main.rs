use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{prelude::*, EnvFilter};

use pclientd::Opt;

#[tokio::main]
async fn main() -> Result<()> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(atty::is(atty::Stream::Stdout))
        .with_target(false);
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);
    registry.init();

    let opt = Opt::parse();

    opt.exec().await
}
