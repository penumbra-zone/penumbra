use clap::Parser;
use elcuity::Opt;
use rustls::crypto::aws_lc_rs;
use std::io::IsTerminal;
use tracing_subscriber::{prelude::*, EnvFilter};

/// Configure tracing_subscriber for logging messages
fn init_tracing() -> anyhow::Result<()> {
    // Instantiate tracing layers.
    // The `FmtLayer` is used to print to the console.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(std::io::stdout().is_terminal())
        .with_writer(std::io::stderr)
        .with_target(true);
    // The `EnvFilter` layer is used to filter events based on `RUST_LOG`.
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    // Register the tracing subscribers.
    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);
    registry.init();
    Ok(())
}

#[tokio::main()]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    init_tracing()?;

    // Parse command line options
    let opt = Opt::parse();

    aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to initialize rustls support, via aws-lc-rs");

    // Run the command
    opt.run().await?;
    Ok(())
}
