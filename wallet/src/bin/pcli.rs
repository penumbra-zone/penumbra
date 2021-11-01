use anyhow::Result;
use std::str::FromStr;
use structopt::StructOpt;

use penumbra_wallet::state;
use tendermint_rpc::{Client, HttpClient, Url};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pcli",
    about = "The Penumbra command-line interface.", 
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {
    /// The address of the Tendermint node.
    #[structopt(short, long, default_value = "127.0.0.1:26658")]
    addr: std::net::SocketAddr,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Creates a transaction.
    Tx { key: String, value: String },
    /// Queries the Penumbra state.
    Query { key: String },
    /// Queries the node for application info.
    Info,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    // The address of the Tendermint node as a `tendermint-rs` `Url`.
    let full_url_str = format!(r#"http://{}"#, opt.addr);
    let url: Url = Url::from_str(&full_url_str[..]).expect("can parse address");

    // xxx If keys exist, load them from disk. If this is first run,
    // we generate keys and start syncing with the chain.
    let _state = state::ClientState::default();
    let client = HttpClient::new(url).unwrap();

    match opt.cmd {
        Command::Tx { key, value } => {
            let rsp = reqwest::get(format!(
                r#"http://{}/broadcast_tx_async?tx="{}={}""#,
                opt.addr, key, value
            ))
            .await?
            .text()
            .await?;

            tracing::info!("{}", rsp);
        }
        Command::Query { key } => {
            let rsp: serde_json::Value = reqwest::get(format!(
                r#"http://{}/abci_query?data=0x{}"#,
                opt.addr,
                hex::encode(key),
            ))
            .await?
            .json()
            .await?;

            tracing::info!(?rsp);
        }
        Command::Info => {
            let net_info = client.net_info().await.unwrap();
            tracing::info!("{:?}", net_info);
        }
    }

    Ok(())
}
