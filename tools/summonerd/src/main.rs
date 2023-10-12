mod coordinator;
mod participant;
mod penumbra_knower;
mod server;
mod storage;

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use console_subscriber::ConsoleLayer;
use coordinator::Coordinator;
use metrics_tracing_context::MetricsLayer;
use penumbra_keys::FullViewingKey;
use penumbra_proto::tools::summoning::v1alpha1::ceremony_coordinator_service_server::CeremonyCoordinatorServiceServer;
use penumbra_proto::tools::summoning::v1alpha1::CeremonyCrs;
use penumbra_proto::Message;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::net::SocketAddr;
use storage::Storage;
use tonic::transport::Server;
use tracing_subscriber::{prelude::*, EnvFilter};
use url::Url;

use crate::{penumbra_knower::PenumbraKnower, server::CoordinatorService};
use penumbra_proof_setup::all::{Phase1CeremonyCRS, Phase1RawCeremonyCRS};

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
    /// Generate a phase 1 root (for testing purposes).
    GeneratePhase1 {
        #[clap(long, display_order = 100)]
        output: Utf8PathBuf,
    },
    /// Initialize the coordinator.
    Init {
        #[clap(long, display_order = 100)]
        storage_dir: Utf8PathBuf,
        #[clap(long, display_order = 200)]
        phase1_root: Utf8PathBuf,
    },
    /// Start the coordinator.
    Start {
        #[clap(long, display_order = 700)]
        storage_dir: Utf8PathBuf,
        #[clap(long, display_order = 800)]
        fvk: FullViewingKey,
        #[clap(long, display_order = 900)]
        node: Url,
        #[clap(long, display_order = 901, default_value = "127.0.0.1:8081")]
        listen: SocketAddr,
    },
}

impl Opt {
    async fn exec(self) -> Result<()> {
        match self.cmd {
            Command::Start {
                storage_dir,
                fvk,
                node,
                listen,
            } => {
                let storage = Storage::load(storage_dir.join("ceremony.db")).await?;
                let knower =
                    PenumbraKnower::load_or_initialize(storage_dir.join("penumbra.db"), &fvk, node)
                        .await?;
                let (coordinator, participant_tx) = Coordinator::new(storage.clone());
                let coordinator_handle = tokio::spawn(coordinator.run());
                let service = CoordinatorService::new(knower, storage, participant_tx);
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
            Command::Init {
                storage_dir,
                phase1_root,
            } => {
                let file = File::open(phase1_root)?;
                let mut reader = BufReader::new(file);

                let mut phase_1_bytes = Vec::new();
                let mut buffer = [0; 4096 * 10]; // 40 KB chunks

                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    phase_1_bytes.extend_from_slice(&buffer[..bytes_read]);
                }
                dbg!("loaded the file");

                let phase_1_raw_root =
                    Phase1RawCeremonyCRS::try_from(CeremonyCrs::decode(&phase_1_bytes[..])?)?;

                dbg!("got phase 1 root");
                // This is assumed to be valid as it's the starting point for the ceremony.
                let phase_1_root = phase_1_raw_root.assume_valid();

                Storage::initialize(storage_dir.join("ceremony.db"), phase_1_root).await?;
                Ok(())
            }
            Command::GeneratePhase1 { output } => {
                let phase_1_root = Phase1CeremonyCRS::root()?;
                let proto_encoded_phase_1_root: CeremonyCrs = phase_1_root.try_into()?;
                std::fs::write(output, proto_encoded_phase_1_root.encode_to_vec())?;
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
