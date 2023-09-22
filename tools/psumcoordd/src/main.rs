mod coordinator;
mod participant;
mod server;
mod storage;

use anyhow::Result;
use clap::Parser;
use coordinator::Coordinator;
use penumbra_proto::{
    tools::summoning::v1alpha1::ceremony_coordinator_service_server::CeremonyCoordinatorServiceServer,
    view::v1alpha1::{
        view_protocol_service_client::ViewProtocolServiceClient,
        view_protocol_service_server::ViewProtocolServiceServer,
    },
};
use penumbra_view::{ViewClient, ViewService};
use std::{net::SocketAddr, time::Duration};
use storage::Storage;
use tokio::time::sleep;
use tonic::transport::Server;

use crate::server::CoordinatorService;

#[derive(Debug, Parser)]
#[clap(
    name = "psumcoordd",
    about = "Penumbra summoning ceremony coordinator",
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {
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
                let storage = Storage::new();
                let (coordinator, participant_tx) = Coordinator::new(storage.clone());
                let coordinator_handle = tokio::spawn(coordinator.run());
                let service = CoordinatorService::new(storage, participant_tx);
                let grpc_server =
                    Server::builder()
                        .accept_http1(true)
                        .add_service(tonic_web::enable(CeremonyCoordinatorServiceServer::new(
                            service,
                        )));
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
    let opt = Opt::parse();

    opt.exec().await
}
