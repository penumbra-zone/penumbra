use clap::Parser;
use elcuity::Opt;
use rustls::crypto::aws_lc_rs;
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

    aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to initialize rustls support, via aws-lc-rs");

    // Run the command
    opt.run().await?;
    Ok(())
}
