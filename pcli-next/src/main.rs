// Rust analyzer complains without this (but rustc is happy regardless)
#![recursion_limit = "256"]
#![allow(clippy::clone_on_copy)]
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use structopt::StructOpt;

mod command;
mod fetch;
mod network;
mod state;
mod sync;
mod wallet;
mod warning;

const WALLET_FILE_PATH: &'static str = "wallet.json";
const VIEW_FILE_PATH: &'static str = "pcli-view.sqlite";

use command::*;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pcli-next",
    about = "The Penumbra command-line interface.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
pub struct Opt {
    /// The address of the pd+tendermint node.
    #[structopt(short, long, default_value = "testnet.penumbra.zone")]
    pub node: String,
    /// The port to use to speak to tendermint's RPC server.
    #[structopt(long, default_value = "26657")]
    pub tendermint_port: u16,
    /// The port to use to speak to pd's gRPC server.
    #[structopt(long, default_value = "8080")]
    pub pd_port: u16,
    #[structopt(subcommand)]
    pub cmd: Command,
    /// The directory to store the wallet and view data in [default: platform appdata directory]
    #[structopt(short, long)]
    pub data_path: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Display a warning message to the user so they don't get upset when all their tokens are lost.
    if std::env::var("PCLI_UNLEASH_DANGER").is_err() {
        warning::display();
    }

    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    let default_data_dir = ProjectDirs::from("zone", "penumbra", "pcli")
        .context("Failed to get platform data dir")?
        .data_dir()
        .to_path_buf();
    let data_dir = opt
        .data_path
        .as_ref()
        .map(|s| PathBuf::from(s))
        .unwrap_or(default_data_dir);

    // Create the data directory if it is missing.
    std::fs::create_dir_all(&data_dir).context("Failed to create data directory")?;

    // The wallet command takes the data dir directly, since it may need to
    // create the client state, so handle it specially here so that we can have
    // common code for the other subcommands.

    if let Command::Wallet(wallet_cmd) = &opt.cmd {
        wallet_cmd.exec(data_dir)?;
        return Ok(());
    }

    let wallet_path = data_dir.join("penumbra_wallet.json");
    let view_path = data_dir.join("pcli_view.sqlite");

    // Otherwise, start the sync

    // Synchronize the wallet if the command requires it to be synchronized before it is run.
    let mut state = ClientStateFile::load(wallet_path.clone())?;

    // Chain params may not have been fetched yet, do so if necessary.
    if state.chain_params().is_none() {
        fetch::chain_params(&opt, &mut state).await?;
    }
    // From now on, we can .expect() on the chain params.

    if opt.cmd.needs_sync() {
        sync(&opt, &mut state).await?;
        fetch::assets(&opt, &mut state).await?;
    };

    match &opt.cmd {
        Command::Wallet(_) => unreachable!("wallet command already executed"),
        Command::Sync => {
            // We have already synchronized the wallet above, so we can just return.
        }
        Command::Tx(tx_cmd) => tx_cmd.exec(&opt, &mut state).await?,
        Command::Addr(addr_cmd) => addr_cmd.exec(&mut state)?,
        Command::Balance(balance_cmd) => balance_cmd.exec(&state)?,
        Command::Validator(cmd) => cmd.exec(&opt, &mut state).await?,
        Command::Stake(cmd) => cmd.exec(&opt, &mut state).await?,
        Command::Tmp(cmd) => cmd.exec().await?,
        Command::Chain(cmd) => cmd.exec(&opt, &state).await?,
    }

    Ok(())
}
