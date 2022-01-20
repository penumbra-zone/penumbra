#![allow(clippy::clone_on_copy)]
use std::path::PathBuf;

use anyhow::Result;
use directories::ProjectDirs;
use structopt::StructOpt;

mod command;
mod fetch;
mod network;
mod state;
mod sync;
mod warning;

use command::*;
use state::ClientStateFile;
use sync::sync;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pcli",
    about = "The Penumbra command-line interface.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
pub struct Opt {
    /// The address of the pd+tendermint node.
    #[structopt(short, long, default_value = "eupheme.penumbra.zone")]
    pub node: String,
    /// The port to use to speak to tendermint.
    #[structopt(short, long, default_value = "26657")]
    pub rpc_port: u16,
    /// The port to use to speak to pd's light wallet server.
    #[structopt(short, long, default_value = "26666")]
    pub light_wallet_port: u16,
    /// The port to use to speak to pd's thin wallet server.
    #[structopt(short, long, default_value = "26667")]
    pub thin_wallet_port: u16,
    #[structopt(subcommand)]
    pub cmd: Command,
    /// The location of the wallet file [default: platform appdata directory]
    #[structopt(short, long)]
    pub wallet_location: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Display a warning message to the user so they don't get upset when all their tokens are lost.
    if std::env::var("PCLI_UNLEASH_DANGER").is_err() {
        warning::display();
    }

    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    let project_dir =
        ProjectDirs::from("zone", "penumbra", "pcli").expect("can access penumbra project dir");
    // Currently we use just the data directory. Create it if it is missing.
    std::fs::create_dir_all(project_dir.data_dir()).expect("can create penumbra data directory");

    // We store wallet data in `penumbra_wallet.dat` in the state directory, unless
    // the user provides another location.
    let wallet_path = opt.wallet_location.as_ref().map_or_else(
        || project_dir.data_dir().join("penumbra_wallet.json"),
        PathBuf::from,
    );

    // The wallet command takes the wallet_path directly, since it may need to create the client state,
    // so handle it specially here so that we can have common code for the other subcommands.
    if let Command::Wallet(wallet_cmd) = &opt.cmd {
        wallet_cmd.exec(wallet_path)?;
        return Ok(());
    }

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
        Command::Validator(cmd) => cmd.exec(&opt, &state).await?,
        Command::Stake(cmd) => cmd.exec(&opt, &mut state).await?,
    }

    Ok(())
}
