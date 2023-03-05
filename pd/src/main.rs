#![allow(clippy::clone_on_copy)]
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use console_subscriber::ConsoleLayer;
use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Stack;

use anyhow::Context;
use clap::{Parser, Subcommand};
use metrics_exporter_prometheus::PrometheusBuilder;
use pd::testnet::{
    generate::testnet_generate, get_testnet_dir, join::testnet_join, parse_tm_address,
};
use penumbra_proto::client::v1alpha1::{
    oblivious_query_service_server::ObliviousQueryServiceServer,
    specific_query_service_server::SpecificQueryServiceServer,
    tendermint_proxy_service_server::TendermintProxyServiceServer,
};
use penumbra_storage::Storage;
use rand::Rng;
use rand_core::OsRng;
use tokio::runtime;
use tonic::transport::Server;
use tracing_subscriber::{prelude::*, EnvFilter};

use tendermint_config::net::Address as TendermintAddress;

#[derive(Debug, Parser)]
#[clap(
    name = "pd",
    about = "The Penumbra daemon.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {
    /// Enable Tokio Console support.
    #[clap(long, default_value = "false")]
    tokio_console: bool,
    /// Command to run.
    #[clap(subcommand)]
    cmd: RootCommand,
}

#[derive(Debug, Subcommand)]
enum RootCommand {
    /// Start running the ABCI and wallet services.
    Start {
        /// The path used to store pd-releated data, including the Rocks database.
        #[clap(long)]
        home: PathBuf,
        /// Bind the services to this host.
        #[clap(long, default_value = "127.0.0.1")]
        host: String,
        /// Bind the ABCI server to this port.
        #[clap(short, long, default_value = "26658")]
        abci_port: u16,
        /// Bind the gRPC server to this port.
        #[clap(short, long, default_value = "8080")]
        grpc_port: u16,
        /// bind the metrics endpoint to this port.
        #[clap(short, long, default_value = "9000")]
        metrics_port: u16,
        /// Proxy Tendermint requests against the gRPC server to this address.
        #[clap(short, long, default_value = "http://127.0.0.1:26657")]
        tendermint_addr: url::Url,
    },
    /// Generate, join, or reset a testnet.
    Testnet {
        /// Path to directory to store output in. Must not exist. Defaults to
        /// ~/.penumbra/testnet_data".
        #[clap(long)]
        testnet_dir: Option<PathBuf>,

        #[clap(subcommand)]
        tn_cmd: TestnetCommand,
    },
}

#[derive(Debug, Subcommand)]
enum TestnetCommand {
    /// Generates a directory structure containing necessary files to create a new
    /// testnet from genesis, based on input configuration.
    Generate {
        /// The `timeout_commit` parameter (block interval) to configure Tendermint with.
        #[clap(long)]
        timeout_commit: Option<tendermint::Timeout>,
        /// Number of blocks per epoch.
        #[clap(long)]
        epoch_duration: Option<u64>,
        /// Number of epochs before unbonding stake is released.
        #[clap(long)]
        unbonding_epochs: Option<u64>,
        /// Maximum number of validators in the consensus set.
        #[clap(long)]
        active_validator_limit: Option<u64>,
        /// Whether to preserve the chain ID (useful for public testnets) or append a random suffix (useful for dev/testing).
        #[clap(long)]
        preserve_chain_id: bool,
        /// Path to CSV file containing initial allocations [default: latest testnet].
        #[clap(long, parse(from_os_str))]
        allocations_input_file: Option<PathBuf>,
        /// Path to JSON file containing initial validator configs [default: latest testnet].
        #[clap(long, parse(from_os_str))]
        validators_input_file: Option<PathBuf>,
        /// Testnet name [default: latest testnet].
        #[clap(long)]
        chain_id: Option<String>,
        /// IP Address to start `tendermint` nodes on. Increments by three to make room for `pd` per node.
        #[clap(long, default_value = "192.167.10.11")]
        starting_ip: Ipv4Addr,
    },

    /// Like `testnet generate`, but joins the testnet to which the specified node belongs
    Join {
        #[clap(default_value = "testnet.penumbra.zone")]
        node: String,
        // Default: node-#
        /// Human-readable name to identify node on network
        // Default: 'node-#'
        #[clap(long)]
        moniker: Option<String>,
        /// Public IP address to advertise for this node's Tendermint P2P service.
        /// Setting this option will instruct other nodes on the network to connect
        /// to yours.
        #[clap(long)]
        external_address: Option<String>,
    },

    /// Reset all `pd` testnet state.
    UnsafeResetAll {},
}

// Extracted from tonic's remote_addr implementation; we'd like to instrument
// spans with the remote addr at the server level rather than at the individual
// request level, but the hook available to do that gives us an http::Request
// rather than a tonic::Request, so the tonic::Request::remote_addr method isn't
// available.
fn remote_addr(req: &http::Request<()>) -> Option<SocketAddr> {
    use tonic::transport::server::TcpConnectInfo;
    // NOTE: needs to also check TlsConnectInfo if we use TLS
    req.extensions()
        .get::<TcpConnectInfo>()
        .and_then(|i| i.remote_addr())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Instantiate tracing layers.
    // The MetricsLayer handles enriching metrics output with labels from tracing spans.
    let metrics_layer = MetricsLayer::new();
    // The ConsoleLayer enables collection of data for `tokio-console`.
    let console_layer = ConsoleLayer::builder().with_default_env().spawn();
    // The `FmtLayer` is used to print to the console.
    let fmt_layer = tracing_subscriber::fmt::layer().with_target(false);
    // The `EnvFilter` layer is used to filter events based on `RUST_LOG`.
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

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

    match opt.cmd {
        RootCommand::Start {
            home,
            host,
            abci_port,
            grpc_port,
            metrics_port,
            tendermint_addr,
        } => {
            tracing::info!(?host, ?abci_port, ?grpc_port, "starting pd");

            let mut rocks_path = home.clone();
            rocks_path.push("rocksdb");

            let storage = Storage::load(rocks_path)
                .await
                .context("Unable to initialize RocksDB storage")?;

            let consensus = pd::Consensus::new(storage.clone()).await?;
            let mempool = pd::Mempool::new(storage.clone()).await?;
            let info = pd::Info::new(storage.clone());
            let tm_proxy = pd::TendermintProxy::new(tendermint_addr);
            let snapshot = pd::Snapshot {};

            let abci_server = tokio::task::Builder::new()
                .name("abci_server")
                .spawn(
                    tower_abci::Server::builder()
                        .consensus(consensus)
                        .snapshot(snapshot)
                        .mempool(mempool)
                        .info(info.clone())
                        .finish()
                        .unwrap()
                        .listen(format!("{host}:{abci_port}")),
                )
                .expect("failed to spawn abci server");

            let grpc_server = tokio::task::Builder::new()
                .name("grpc_server")
                .spawn(
                    Server::builder()
                        .trace_fn(|req| match remote_addr(req) {
                            Some(remote_addr) => {
                                tracing::error_span!("grpc", ?remote_addr)
                            }
                            None => tracing::error_span!("grpc"),
                        })
                        // Allow HTTP/1, which will be used by grpc-web connections.
                        .accept_http1(true)
                        // Wrap each of the gRPC services in a tonic-web proxy:
                        .add_service(tonic_web::enable(ObliviousQueryServiceServer::new(
                            info.clone(),
                        )))
                        .add_service(tonic_web::enable(SpecificQueryServiceServer::new(
                            info.clone(),
                        )))
                        .add_service(tonic_web::enable(TendermintProxyServiceServer::new(
                            tm_proxy.clone(),
                        )))
                        .serve(
                            format!("{host}:{grpc_port}")
                                .parse()
                                .expect("this is a valid address"),
                        ),
                )
                .expect("failed to spawn grpc server");

            // Configure a Prometheus recorder and exporter.
            let (recorder, exporter) = PrometheusBuilder::new()
                .with_http_listener(
                    format!("{host}:{metrics_port}")
                        .parse::<SocketAddr>()
                        .expect("this is a valid address"),
                )
                .build()
                .expect("failed to build prometheus recorder");

            Stack::new(recorder)
                // Adding the `TracingContextLayer` will add labels from the tracing span to metrics.
                // The only labels to be included are "chain_id" and "role".
                .push(TracingContextLayer::only_allow(["chain_id", "role"]))
                .install()
                .expect("global recorder already installed");

            // This spawns the HTTP service that lets Prometheus pull metrics from `pd`
            let handle = runtime::Handle::try_current().expect("unable to get runtime handle");
            handle.spawn(exporter);

            pd::register_metrics();

            // TODO: better error reporting
            // We error out if a service errors, rather than keep running
            tokio::select! {
                x = abci_server => x?.map_err(|e| anyhow::anyhow!(e))?,
                x = grpc_server => x?.map_err(|e| anyhow::anyhow!(e))?,
            };
        }

        RootCommand::Testnet {
            tn_cmd: TestnetCommand::UnsafeResetAll {},
            testnet_dir,
        } => {
            let testnet_dir = get_testnet_dir(testnet_dir);
            if testnet_dir.exists() {
                std::fs::remove_dir_all(testnet_dir)?;
            } else {
                tracing::info!(
                    "Testnet directory does not exist, so not removing: {}",
                    testnet_dir.display()
                );
            }
        }

        RootCommand::Testnet {
            tn_cmd:
                TestnetCommand::Join {
                    node,
                    moniker,
                    external_address,
                },
            testnet_dir,
        } => {
            let output_dir = get_testnet_dir(testnet_dir);

            // If the output directory already exists, bail out, rather than overwriting.
            if output_dir.exists() {
                return Err(anyhow::anyhow!(
                    "output directory {:?} already exists, refusing to overwrite it",
                    output_dir
                ));
            }

            // Check whether an external address was set, and parse as TendermintAddress.
            let external_address: Option<TendermintAddress> = match external_address {
                Some(a) => parse_tm_address(None, &a).ok(),
                None => None,
            };

            // Set custom moniker, or default to random string suffix.
            let node_name = match moniker {
                Some(m) => m,
                None => format!("node-{}", hex::encode(OsRng.gen::<u32>().to_le_bytes())),
            };

            // Join the target testnet, looking up network info and writing
            // local configs for pd and tendermint.
            testnet_join(output_dir, &node, &node_name, external_address).await?;
        }

        RootCommand::Testnet {
            tn_cmd:
                TestnetCommand::Generate {
                    // TODO this config is gated on a "populate persistent peers"
                    // setting in the Go tendermint binary. Populating the persistent
                    // peers will be useful in local setups until peer discovery via a seed
                    // works.
                    starting_ip,
                    timeout_commit,
                    epoch_duration,
                    unbonding_epochs,
                    active_validator_limit,
                    allocations_input_file,
                    validators_input_file,
                    chain_id,
                    preserve_chain_id,
                },
            testnet_dir,
        } => {
            // Build script computes the latest testnet name and sets it as an env variable
            let chain_id = match preserve_chain_id {
                true => chain_id.unwrap_or_else(|| env!("PD_LATEST_TESTNET_NAME").to_string()),
                false => {
                    // If preserve_chain_id is false, we append a random suffix to avoid collisions
                    let randomizer = OsRng.gen::<u32>();
                    let chain_id =
                        chain_id.unwrap_or_else(|| env!("PD_LATEST_TESTNET_NAME").to_string());
                    format!("{}-{}", chain_id, hex::encode(randomizer.to_le_bytes()))
                }
            };

            let output_dir = get_testnet_dir(testnet_dir);
            // If the output directory already exists, bail out, rather than overwriting.
            if output_dir.exists() {
                return Err(anyhow::anyhow!(
                    "output directory {:?} already exists, refusing to overwrite it",
                    output_dir
                ));
            }

            // Build and write local configs based on input flags.
            testnet_generate(
                output_dir,
                &chain_id,
                active_validator_limit,
                timeout_commit,
                epoch_duration,
                unbonding_epochs,
                starting_ip,
                validators_input_file,
                allocations_input_file,
            )?;
        }
    }
    Ok(())
}
