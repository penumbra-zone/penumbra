mod coordinator;
mod participant;
mod server;
mod storage;

use anyhow::Result;
use clap::Parser;
use console_subscriber::ConsoleLayer;
use coordinator::Coordinator;
use metrics_tracing_context::MetricsLayer;
use penumbra_proof_setup::all::Phase2CeremonyCRS;
use penumbra_proto::tools::summoning::v1alpha1::ceremony_coordinator_service_server::CeremonyCoordinatorServiceServer;
use std::net::SocketAddr;
use storage::Storage;
use tonic::transport::Server;
use tracing_subscriber::{prelude::*, EnvFilter};

use crate::server::CoordinatorService;

/// 100 MIB
const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024;

#[derive(Debug, Parser)]
#[clap(
    name = "summonerd",
    about = "Penumbra summoning ceremony coordinator",
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {
    /// Enable Tokio Console support.
    #[clap(long, default_value = "false")]
    tokio_console: bool,
    /// Command to run.
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Start the coordinator.
    Start {
        #[clap(long, display_order = 900, default_value = "127.0.0.1:8081")]
        listen: SocketAddr,
    },
}

impl Opt {
    async fn exec(self) -> Result<()> {
        match self.cmd {
            Command::Start { listen } => {
                let root = Phase2CeremonyCRS::root()?;
                let storage = Storage::new(root);
                let (coordinator, participant_tx) = Coordinator::new(storage.clone());
                let coordinator_handle = tokio::spawn(coordinator.run());
                let service = CoordinatorService::new(storage, participant_tx);
                let grpc_server =
                    Server::builder()
                        .accept_http1(true)
                        .add_service(tonic_web::enable(
                            CeremonyCoordinatorServiceServer::new(service)
                                .max_encoding_message_size(MAX_MESSAGE_SIZE)
                                .max_decoding_message_size(MAX_MESSAGE_SIZE),
                        ));
                tracing::info!(?listen, "starting grpc server");
                let server_handle = tokio::spawn(grpc_server.serve(listen));
                // TODO: better error reporting
                // We error out if a service errors, rather than keep running
                tokio::select! {
                    x = coordinator_handle => x?.map_err(|e| anyhow::anyhow!(e))?,
                    x = server_handle => x?.map_err(|e| anyhow::anyhow!(e))?,
                };
                Ok(())
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Instantiate tracing layers.
    // The MetricsLayer handles enriching metrics output with labels from tracing spans.
    let metrics_layer = MetricsLayer::new();
    // The ConsoleLayer enables collection of data for `tokio-console`.
    let console_layer = ConsoleLayer::builder().with_default_env().spawn();
    // The `FmtLayer` is used to print to the console.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(atty::is(atty::Stream::Stdout))
        .with_target(true);
    // The `EnvFilter` layer is used to filter events based on `RUST_LOG`.
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))?
        .add_directive("r1cs=off".parse().unwrap());

    let opt = Opt::parse();

    // Register the tracing subscribers, conditionally enabling tokio console support
    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(metrics_layer);
    if opt.tokio_console {
        registry.with(console_layer).init();
    } else {
        registry.init();
    }

    opt.exec().await
}
