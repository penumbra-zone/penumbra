#![allow(clippy::clone_on_copy)]
use anyhow::Result;
use std::env;
use std::path::PathBuf;
use structopt::StructOpt;
#[derive(Debug, StructOpt)]
#[structopt(
    name = "pwalletd",
    about = "The Penumbra wallet daemon.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {
    /// Command to run.
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Start running the wallet daemon.
    Init {
        /// The path used to store the SQLite state database.
        #[structopt(short, long)]
        sqlite_path: PathBuf,
        /// The full viewing key
        #[structopt(short, long)]
        fvk: String,
    },
    Start {
        /// Bind the services to this host.
        #[structopt(short, long, default_value = "127.0.0.1")]
        host: String,
        /// Bind the wallet gRPC server to this port.
        #[structopt(short, long, default_value = "8081")]
        wallet_port: u16,
        /// The address of the pd+tendermint node.
        #[structopt(short, long, default_value = "testnet.penumbra.zone")]
        node: String,
        /// The port to use to speak to pd.
        #[structopt(short, long, default_value = "8080")]
        pd_port: u16,
    },
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    match opt.cmd {
        Command::Init {
            sqlite_path: _,
            fvk: _,
        } => todo!(),
        Command::Start {
            host,
            wallet_port,
            node,
            pd_port,
        } => {
            tracing::info!(?host, ?wallet_port, ?node, ?pd_port, "starting pwalletd");

            unimplemented!();
        }
    }
}
