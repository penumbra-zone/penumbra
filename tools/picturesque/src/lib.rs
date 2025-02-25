use clap::Parser;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

mod cometbft;
mod pcli;
mod pd;
pub mod postgres;

#[derive(Clone, Debug)]
struct Context {
    directory: PathBuf,
    log_level: tracing::Level,
}

impl Context {
    fn new(path: &Path, log_level: tracing::Level) -> anyhow::Result<Self> {
        let directory = std::path::absolute(path)?;
        Ok(Self {
            directory,
            log_level,
        })
    }
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
        #[clap(long, default_value = "50")]
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
    #[clap(long, env = "RUST_LOG", default_value = "info")]
    pub log_level: tracing::Level,
    #[clap(subcommand)]
    pub command: Command,
}

impl Options {
    fn context(&self) -> anyhow::Result<Context> {
        Context::new(self.directory.as_path(), self.log_level)
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let ctx = self.context()?;
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
    let postgres = tokio::spawn(postgres::run(ctx.directory.clone()));
    let postgres_connection_string = postgres::go_connection_string(&ctx.directory);
    let cometbft = tokio::spawn(cometbft::run(
        ctx.directory.clone(),
        postgres_connection_string,
        Some(Duration::from_millis(2000)),
    ));
    let pd = tokio::spawn(pd::run(
        ctx.directory.clone(),
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

#[tracing::instrument(skip(ctx))]
async fn create_devnet(ctx: Context, epoch_duration: u32) -> anyhow::Result<()> {
    tracing::info!("creating devnet");
    pcli::init_with_test_keys(ctx.directory.clone()).await?;
    pd::generate(ctx.directory, ctx.log_level, epoch_duration).await?;
    Ok(())
}
