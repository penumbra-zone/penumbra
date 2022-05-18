#![allow(clippy::clone_on_copy)]
use anyhow::Result;
use penumbra_crypto::FullViewingKey;
use penumbra_proto::client::oblivious::oblivious_query_client::ObliviousQueryClient;
use penumbra_proto::client::oblivious::ChainParamsRequest;
use penumbra_proto::wallet::wallet_protocol_server::WalletProtocolServer;
use penumbra_wallet_next::WalletService;
use std::env;
use std::str::FromStr;
use structopt::StructOpt;
use tonic::transport::Server;
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
    /// The path used to store the SQLite state database.
    #[structopt(short, long, default_value = "pwalletd-db.sqlite")]
    sqlite_path: String,
    /// The address of the pd+tendermint node.
    #[structopt(short, long, default_value = "testnet.penumbra.zone")]
    node: String,
    /// The port to use to speak to tendermint's RPC server.
    #[structopt(long, default_value = "26657")]
    tendermint_port: u16,
    /// The port to use to speak to pd's gRPC server.
    #[structopt(long, default_value = "8080")]
    pd_port: u16,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Start running the wallet daemon.
    Init {
        /// The full viewing key
        #[structopt(short, long)]
        full_viewing_key: String,
    },
    Start {
        /// Bind the services to this host.
        #[structopt(short, long, default_value = "127.0.0.1")]
        host: String,
        /// Bind the wallet gRPC server to this port.
        #[structopt(long, default_value = "8081")]
        wallet_port: u16,
    },
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    let mut client =
        ObliviousQueryClient::connect(format!("http://{}:{}", opt.node, opt.pd_port)).await?;

    match opt.cmd {
        Command::Init { full_viewing_key } => {
            let params = client
                .chain_params(tonic::Request::new(ChainParamsRequest {
                    chain_id: String::new(),
                }))
                .await?
                .into_inner()
                .try_into()?;
            penumbra_wallet_next::Storage::initialize(
                opt.sqlite_path,
                FullViewingKey::from_str(full_viewing_key.as_ref())?,
                params,
            )
            .await?;
            Ok(())
        }
        Command::Start { host, wallet_port } => {
            tracing::info!(?opt.sqlite_path, ?host, ?wallet_port, ?opt.node, ?opt.tendermint_port, ?opt.pd_port, "starting pwalletd");

            let storage = penumbra_wallet_next::Storage::load(opt.sqlite_path).await?;

            let service = WalletService::new(storage, client).await?;

            tokio::task::Builder::new()
                .name("wallet_grpc_server")
                .spawn(
                    Server::builder()
                        .add_service(WalletProtocolServer::new(service))
                        .serve(
                            format!("{}:{}", host, wallet_port)
                                .parse()
                                .expect("this is a valid address"),
                        ),
                )
                .await?
                .map_err(|_| anyhow::anyhow!("tonic transport error on start"))
        }
    }
}
