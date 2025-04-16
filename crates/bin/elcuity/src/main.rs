use clap::Parser;
use elcuity::Opt;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main()]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // Parse command line options
    let opt = Opt::parse();

    // Run the command
    opt.run().await?;
    Ok(())
}
