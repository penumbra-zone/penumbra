#![allow(clippy::clone_on_copy)]
use anyhow::Result;
use penumbra_chain::params::ChainParams;
use penumbra_crypto::FullViewingKey;
use penumbra_proto::client::oblivious::oblivious_query_client::ObliviousQueryClient;
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
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Start running the wallet daemon.
    Init {
        /// The path used to store the SQLite state database.
        #[structopt(short, long)]
        sqlite_path: String,
        /// The full viewing key
        #[structopt(short, long)]
        fvk: String,
    },
    Start {
        /// The path used to store the SQLite state database.
        #[structopt(short, long)]
        sqlite_path: String,
        /// Bind the services to this host.
        #[structopt(short, long, default_value = "127.0.0.1")]
        host: String,
        /// Bind the wallet gRPC server to this port.
        #[structopt(short, long, default_value = "8081")]
        wallet_port: u16,
    },
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    match opt.cmd {
        Command::Init { sqlite_path, fvk } => {
            penumbra_wallet_next::Storage::initialize(
                sqlite_path,
                FullViewingKey::from_str(fvk.as_ref())?,
                ChainParams::default(),
            )
            .await?;
            Ok(())
        }
        Command::Start {
            sqlite_path,
            host,
            wallet_port,
        } => {
            tracing::info!(?sqlite_path, ?host, ?wallet_port, "starting pwalletd");

            let storage = penumbra_wallet_next::Storage::load(sqlite_path).await?;
            let client =
                ObliviousQueryClient::connect(format!("http://{}:{}", host, wallet_port)).await?;
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
                );

            todo!()
        }
    }
}
