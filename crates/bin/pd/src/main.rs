#![allow(clippy::clone_on_copy)]
#![deny(clippy::unwrap_used)]
#![recursion_limit = "512"]

use pd::cli::Opt;

/// The `pd` daemon's entrypoint.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Validate options immediately.
    let Opt { tokio_console, cmd } = <Opt as clap::Parser>::parse();

    // Instantiate tracing layers.
    pd::tracing::init(tokio_console)?;

    // Run the command.
    cmd.run().await
}
