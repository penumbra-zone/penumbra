#![allow(clippy::clone_on_copy)]
#![recursion_limit = "512"]
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use console_subscriber::ConsoleLayer;
use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Stack;

use anyhow::Context;
use clap::{Parser, Subcommand};
use futures::stream::TryStreamExt;
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
use penumbra_tendermint_proxy::TendermintProxy;
use penumbra_tower_trace::remote_addr;
use rand::Rng;
use rand_core::OsRng;
use tendermint::abci::{ConsensusRequest, MempoolRequest};
use tendermint_config::net::Address as TendermintAddress;
use tokio::{net::TcpListener, runtime};
use tonic::transport::Server;
use tracing_subscriber::{prelude::*, EnvFilter};
use url::Url;

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
    /// Starts the Penumbra daemon.
    Start {
        /// The path used to store all `pd`-related data and configuration.
        #[clap(long, env = "PENUMBRA_PD_HOME", display_order = 100)]
        home: PathBuf,
        /// Bind the ABCI server to this socket.
        ///
        /// The ABCI server is used by Tendermint to drive the application state.
        #[clap(
            short,
            long,
            env = "PENUMBRA_PD_ABCI_BIND",
            default_value = "127.0.0.1:26658",
            display_order = 400
        )]
        abci_bind: SocketAddr,
        /// Bind the gRPC server to this socket.
        ///
        /// The gRPC server supports both grpc (HTTP/2) and grpc-web (HTTP/1.1) clients.
        ///
        /// If `grpc_auto_https` is set, this defaults to `0.0.0.0:443` and uses HTTPS.
        ///
        /// If `grpc_auto_https` is not set, this defaults to `127.0.0.1:8080` without HTTPS.
        #[clap(short, long, env = "PENUMBRA_PD_GRPC_BIND", display_order = 201)]
        grpc_bind: Option<SocketAddr>,
        /// If set, serve gRPC using auto-managed HTTPS with this domain name.
        ///
        /// NOTE: This option automatically provisions TLS certificates from
        /// Let's Encrypt and caches them in the `home` directory.  The
        /// production LE CA has rate limits, so be careful using this option
        /// with `pd testnet unsafe-reset-all`, which will delete the certificates
        /// and force re-issuance, possibly hitting the rate limit.
        #[clap(long, value_name = "DOMAIN", display_order = 200)]
        grpc_auto_https: Option<String>,
        /// Bind the metrics endpoint to this socket.
        #[clap(
            short,
            long,
            env = "PENUMBRA_PD_METRICS_BIND",
            default_value = "127.0.0.1:9000",
            display_order = 300
        )]
        metrics_bind: SocketAddr,
        /// The JSON-RPC address of the Tendermint node driving this `pd`
        /// instance.
        ///
        /// This is used to proxy requests from the gRPC server to Tendermint,
        /// so clients only need to connect to one endpoint and don't need to
        /// worry about the peculiarities of Tendermint's JSON-RPC encoding
        /// format.
        #[clap(
            short,
            long,
            env = "PENUMBRA_PD_TM_PROXY_URL",
            default_value = "http://127.0.0.1:26657",
            display_order = 401
        )]
        tendermint_addr: Url,
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
        /// URL of the remote Tendermint RPC endpoint for bootstrapping connection.
        #[clap(
            env = "PENUMBRA_PD_JOIN_URL",
            default_value = "http://testnet.penumbra.zone:26657"
        )]
        node: Url,
        /// Human-readable name to identify node on network
        // Default: 'node-#'
        #[clap(long, env = "PENUMBRA_PD_TM_MONIKER")]
        moniker: Option<String>,
        /// Public URL to advertise for this node's Tendermint P2P service.
        /// Setting this option will instruct other nodes on the network to connect
        /// to yours. Must be in the form of a socket, e.g. "1.2.3.4:26656".
        #[clap(long, env = "PENUMBRA_PD_TM_EXTERNAL_ADDR")]
        external_address: Option<SocketAddr>,
        /// When generating Tendermint config, use this socket to bind the Tendermint RPC service.
        #[clap(long, env = "PENUMBRA_PD_TM_RPC_BIND", default_value = "0.0.0.0:26657")]
        tendermint_rpc_bind: SocketAddr,
        /// When generating Tendermint config, use this socket to bind the Tendermint P2P service.
        #[clap(long, env = "PENUMBRA_PD_TM_P2P_BIND", default_value = "0.0.0.0:26656")]
        tendermint_p2p_bind: SocketAddr,
    },

    /// Reset all `pd` testnet state.
    UnsafeResetAll {},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Instantiate tracing layers.
    // The MetricsLayer handles enriching metrics output with labels from tracing spans.
    let metrics_layer = MetricsLayer::new();
    // The ConsoleLayer enables collection of data for `tokio-console`.
    let console_layer = ConsoleLayer::builder().with_default_env().spawn();
    // The `FmtLayer` is used to print to the console.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(atty::is(atty::Stream::Stdout))
        .with_target(false);
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
            abci_bind,
            grpc_bind,
            grpc_auto_https,
            metrics_bind,
            tendermint_addr,
        } => {
            tracing::info!(
                ?abci_bind,
                ?grpc_bind,
                ?grpc_auto_https,
                ?metrics_bind,
                "starting pd"
            );

            let mut rocks_path = home.clone();
            rocks_path.push("rocksdb");

            let storage = Storage::load(rocks_path)
                .await
                .context("Unable to initialize RocksDB storage")?;

            use penumbra_tower_trace::trace::request_span;
            use penumbra_tower_trace::RequestExt;

            let consensus = tower::ServiceBuilder::new()
                .layer(request_span::layer(|req: &ConsensusRequest| {
                    req.create_span()
                }))
                .service(tower_actor::Actor::new(10, |queue: _| {
                    let storage = storage.clone();
                    async move {
                        pd::Consensus::new(storage.clone(), queue)
                            .await?
                            .run()
                            .await
                    }
                }));
            let mempool = tower::ServiceBuilder::new()
                .layer(request_span::layer(|req: &MempoolRequest| {
                    req.create_span()
                }))
                .service(tower_actor::Actor::new(10, |queue: _| {
                    let storage = storage.clone();
                    async move { pd::Mempool::new(storage.clone(), queue).await?.run().await }
                }));
            let info = pd::Info::new(storage.clone());
            let tm_proxy = TendermintProxy::new(tendermint_addr);
            let snapshot = pd::Snapshot {};

            let abci_server = tokio::task::Builder::new()
                .name("abci_server")
                .spawn(
                    tower_abci::v034::Server::builder()
                        .consensus(consensus)
                        .snapshot(snapshot)
                        .mempool(mempool)
                        .info(info.clone())
                        .finish()
                        .unwrap()
                        .listen(abci_bind),
                )
                .expect("failed to spawn abci server");

            let grpc_server = Server::builder()
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
                .add_service(tonic_web::enable(
                    tonic_reflection::server::Builder::configure()
                        .register_encoded_file_descriptor_set(penumbra_proto::FILE_DESCRIPTOR_SET)
                        .build()
                        .with_context(|| "could not configure grpc reflection service")?,
                ));

            let grpc_server = if let Some(domain) = grpc_auto_https {
                use pd::auto_https::Wrapper;
                use rustls_acme::{caches::DirCache, AcmeConfig};
                use tokio_stream::wrappers::TcpListenerStream;
                use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

                let mut acme_cache = home.clone();
                acme_cache.push("rustls_acme_cache");

                let grpc_bind = grpc_bind.unwrap_or("0.0.0.0:443".parse().unwrap());
                let listener = TcpListenerStream::new(TcpListener::bind(grpc_bind).await?);
                // Configure HTTP2 support for the TLS negotiation; we also permit HTTP1.1
                // for backwards-compatibility, specifically for grpc-web.
                let alpn_config = vec!["h2".into(), "http/1.1".into()];
                let tls_incoming = AcmeConfig::new([domain.as_str()])
                    .cache(DirCache::new(acme_cache))
                    .directory_lets_encrypt(true) // Use the production LE environment
                    .incoming(listener.map_ok(|conn| conn.compat()), alpn_config)
                    .map_ok(|incoming| Wrapper {
                        inner: incoming.compat(),
                    });

                tokio::task::Builder::new()
                    .name("grpc_server")
                    .spawn(grpc_server.serve_with_incoming(tls_incoming))
                    .expect("failed to spawn grpc server")
            } else {
                let grpc_bind = grpc_bind.unwrap_or("127.0.0.1:8080".parse().unwrap());
                tokio::task::Builder::new()
                    .name("grpc_server")
                    .spawn(grpc_server.serve(grpc_bind))
                    .expect("failed to spawn grpc server")
            };

            // Configure a Prometheus recorder and exporter.
            let (recorder, exporter) = PrometheusBuilder::new()
                .with_http_listener(metrics_bind)
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
                    tendermint_rpc_bind,
                    tendermint_p2p_bind,
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
                Some(a) => {
                    let u = Url::parse(format!("tcp://{}", a).as_str())?;
                    parse_tm_address(None, &u).ok()
                }
                None => None,
            };

            // Set custom moniker, or default to random string suffix.
            let node_name = match moniker {
                Some(m) => m,
                None => format!("node-{}", hex::encode(OsRng.gen::<u32>().to_le_bytes())),
            };

            // Join the target testnet, looking up network info and writing
            // local configs for pd and tendermint.
            testnet_join(
                output_dir,
                node,
                &node_name,
                external_address,
                tendermint_rpc_bind,
                tendermint_p2p_bind,
            )
            .await?;
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
