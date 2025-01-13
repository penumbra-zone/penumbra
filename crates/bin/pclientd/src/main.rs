#![deny(clippy::unwrap_used)]
use std::io::IsTerminal as _;

use anyhow::Result;
use clap::Parser;
use rustls::crypto::aws_lc_rs;
use tracing_subscriber::{prelude::*, EnvFilter};

use pclientd::Opt;

#[tokio::main]
async fn main() -> Result<()> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(std::io::stdout().is_terminal())
        .with_target(true);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))?
        // Force disabling of r1cs log messages, otherwise the `ark-groth16` crate
        // can use a massive (>16GB) amount of memory generating unused trace statements.
        .add_directive("r1cs=off".parse()?);
    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);
    registry.init();

    let opt = Opt::parse();

    // Initialize HTTPS support
    // rustls::crypto::aws_lc_rs::default_provider().install_default();
    aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to initialize rustls support, via aws-lc-rs");

    opt.exec().await
}
