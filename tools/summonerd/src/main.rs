mod coordinator;
mod participant;
mod penumbra_knower;
mod server;
mod storage;

use anyhow::Result;
use ark_groth16::{ProvingKey, VerifyingKey};
use ark_serialize::CanonicalSerialize;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use clap::Parser;
use console_subscriber::ConsoleLayer;
use coordinator::Coordinator;
use decaf377::Bls12_377;
use metrics_tracing_context::MetricsLayer;
use penumbra_keys::FullViewingKey;
use penumbra_proof_params::{ProvingKeyExt, VerifyingKeyExt};
use penumbra_proof_setup::all::combine;
use penumbra_proof_setup::all::transition;
use penumbra_proto::tools::summoning::v1alpha1::ceremony_coordinator_service_server::CeremonyCoordinatorServiceServer;
use penumbra_proto::tools::summoning::v1alpha1::CeremonyCrs;
use penumbra_proto::Message;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
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

// To avoid repeating the constant
fn ceremony_db(path: &Utf8Path) -> Utf8PathBuf {
    path.join("ceremony.db")
}

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
    /// Transition between phases
    Transition {
        #[clap(long, display_order = 100)]
        storage_dir: Utf8PathBuf,
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
    /// Export the output of the ceremony
    Export {
        #[clap(long, display_order = 100)]
        storage_dir: Utf8PathBuf,
        #[clap(long, display_order = 200)]
        target_dir: Utf8PathBuf,
    },
}

impl Opt {
    async fn exec(self) -> Result<()> {
        match self.cmd {
            Command::GeneratePhase1 { output } => {
                let phase_1_root = Phase1CeremonyCRS::root()?;
                let proto_encoded_phase_1_root: CeremonyCrs = phase_1_root.try_into()?;
                std::fs::write(output, proto_encoded_phase_1_root.encode_to_vec())?;
                Ok(())
            }
            Command::Start {
                storage_dir,
                fvk,
                node,
                listen,
            } => {
                let storage = Storage::load_or_initialize(ceremony_db(&storage_dir)).await?;
                // Check if we've transitioned, for a nice error message
                if storage.transition_extra_information().await?.is_none() {
                    anyhow::bail!("Please run the transition command before this command 8^)");
                }
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
                reader.read_to_end(&mut phase_1_bytes)?;

                let phase_1_raw_root = Phase1RawCeremonyCRS::unchecked_from_protobuf(
                    CeremonyCrs::decode(&phase_1_bytes[..])?,
                )?;

                // This is assumed to be valid as it's the starting point for the ceremony.
                let phase_1_root = phase_1_raw_root.assume_valid();

                let mut storage = Storage::load_or_initialize(ceremony_db(&storage_dir)).await?;
                storage.set_root(phase_1_root).await?;

                Ok(())
            }
            Command::Transition { storage_dir } => {
                let mut storage = Storage::load_or_initialize(ceremony_db(&storage_dir)).await?;

                let phase1_crs = match storage.phase1_current_crs().await? {
                    Some(x) => x,
                    None => anyhow::bail!("Please run phase1 before this command 8^)"),
                };
                let (aux, phase2_root) = transition(&phase1_crs)?;
                storage.set_transition(phase2_root, aux).await?;

                Ok(())
            }
            Command::Export {
                storage_dir,
                target_dir,
            } => {
                let storage = Storage::load_or_initialize(ceremony_db(&storage_dir)).await?;
                // Grab phase1 output
                let phase1_crs = match storage.phase1_current_crs().await? {
                    Some(x) => x,
                    None => anyhow::bail!("Please run phase1 before this command 8^)"),
                };
                // Grab phase2 output
                let phase2_crs = match storage.phase2_current_crs().await? {
                    Some(x) => x,
                    None => anyhow::bail!("Please run phase2 before this command 8^)"),
                };
                // Grab aux information
                let aux = match storage.transition_extra_information().await? {
                    Some(x) => x,
                    None => anyhow::bail!("Please run phase2 before this command 8^)"),
                };
                let pks = combine(&phase1_crs, &phase2_crs, &aux);
                let names = [
                    "spend",
                    "output",
                    "delegator_vote",
                    "undelegateclaim",
                    "swap",
                    "swapclaim",
                    "nullifier_derivation",
                ];
                for i in 0..7 {
                    write_params(target_dir.as_path(), names[i], &pks[i], &pks[i].vk)?;
                }
                Ok(())
            }
        }
    }
}

fn write_params(
    target_dir: &Utf8Path,
    name: &str,
    pk: &ProvingKey<Bls12_377>,
    vk: &VerifyingKey<Bls12_377>,
) -> Result<()> {
    let pk_location = target_dir.join(format!("{}_pk.bin", name));
    let vk_location = target_dir.join(format!("{}_vk.param", name));
    let id_location = target_dir.join(format!("{}_id.rs", name));

    let pk_file = fs::File::create(&pk_location)?;
    let vk_file = fs::File::create(&vk_location)?;

    let pk_writer = BufWriter::new(pk_file);
    let vk_writer = BufWriter::new(vk_file);

    ProvingKey::serialize_uncompressed(pk, pk_writer).expect("can serialize ProvingKey");
    VerifyingKey::serialize_uncompressed(vk, vk_writer).expect("can serialize VerifyingKey");

    let pk_id = pk.debug_id();
    let vk_id = vk.debug_id();
    std::fs::write(
        id_location,
        format!(
            r#"
pub const PROVING_KEY_ID: &'static str = "{pk_id}";
pub const VERIFICATION_KEY_ID: &'static str = "{vk_id}";
"#,
        ),
    )?;

    Ok(())
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
