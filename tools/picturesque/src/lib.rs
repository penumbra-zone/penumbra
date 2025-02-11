use clap::Parser;
use std::{path::PathBuf, time::Duration};

mod cometbft;
mod pd;
mod postgres;

#[derive(Clone, Debug)]
struct Context {
    directory: PathBuf,
    log_level: tracing::Level,
}

#[derive(Clone, Debug, Parser)]
pub enum Command {
    /// Create a new blank slate for the devnet.
    ///
    /// This will have no permanent state, but be ready to edit for changing params.
    ///
    /// In most cases, you'll want to edit it, and then save a copy for later,
    /// before then spinning up the devnet.
    Create {
        /// How long epochs last, in blocks.
        #[clap(long)]
        epoch_duration: u32,
    },
    /// Start up a devnet.
    ///
    /// The working directory should have been initialized with the `create` command before.
    Start,
}

/// This struct represents the command-line options
#[derive(Clone, Debug, Parser)]
#[clap(name = "picturesque", about = "a simple tool to run devnets", version)]
pub struct Options {
    /// The base of operations for the tool.
    #[clap(short, long)]
    pub directory: PathBuf,
    /// The log level for this program, and the child processes.
    #[clap(long, env = "RUST_LOG")]
    pub log_level: tracing::Level,
    #[clap(subcommand)]
    pub command: Command,
}

impl Options {
    fn context(&self) -> Context {
        Context {
            directory: self.directory.clone(),
            log_level: self.log_level,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let ctx = self.context();
        match self.command {
            Command::Create { epoch_duration } => create_devnet(ctx, epoch_duration).await?,
            Command::Start => run_devnet(ctx).await?,
        }
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
async fn run_devnet(ctx: Context) -> anyhow::Result<()> {
    tracing::info!("spawning postgres");
    let root = ctx.directory.canonicalize()?;
    let postgres = tokio::spawn(postgres::run(root.clone()));
    let cometbft = tokio::spawn(cometbft::run(
        root.clone(),
        Some(Duration::from_millis(2000)),
    ));
    let pd = tokio::spawn(pd::run(
        root.clone(),
        ctx.log_level,
        Some(Duration::from_millis(4000)),
    ));
    tokio::select! {
        x = postgres => x??,
        x = cometbft => x??,
        x = pd => x??,
    };
    Ok(())
}

#[tracing::instrument(skip(_ctx))]
async fn create_devnet(_ctx: Context, _epoch_duration: u32) -> anyhow::Result<()> {
    todo!()
}
