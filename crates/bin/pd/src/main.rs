#![allow(clippy::clone_on_copy)]
#![deny(clippy::unwrap_used)]
#![recursion_limit = "512"]
use std::{net::SocketAddr, path::PathBuf};

use console_subscriber::ConsoleLayer;
use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Stack;

use anyhow::Context;
use clap::{Parser, Subcommand};
use cnidarium::{StateDelta, Storage};
use futures::stream::TryStreamExt;
use ibc_proto::ibc::core::channel::v1::query_server::QueryServer as ChannelQueryServer;
use ibc_proto::ibc::core::client::v1::query_server::QueryServer as ClientQueryServer;
use ibc_proto::ibc::core::connection::v1::query_server::QueryServer as ConnectionQueryServer;
use metrics_exporter_prometheus::PrometheusBuilder;
use pd::events::EventIndexLayer;
use pd::testnet::{
    config::{get_testnet_dir, parse_tm_address, url_has_necessary_parts},
    generate::TestnetConfig,
    join::testnet_join,
};
use pd::upgrade;
use penumbra_app::SUBSTORE_PREFIXES;
use penumbra_proto::core::component::dex::v1alpha1::simulation_service_server::SimulationServiceServer;
use penumbra_proto::util::tendermint_proxy::v1alpha1::tendermint_proxy_service_server::TendermintProxyServiceServer;
use penumbra_tendermint_proxy::TendermintProxy;
use penumbra_tower_trace::remote_addr;
use rand::Rng;
use rand_core::OsRng;
use tendermint_config::net::Address as TendermintAddress;
use tokio::{net::TcpListener, runtime};
use tonic::transport::Server;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{prelude::*, EnvFilter};
use url::Url;

use penumbra_tower_trace::v037::RequestExt;
use tendermint::v0_37::abci::{ConsensusRequest, MempoolRequest};

#[derive(Debug, Parser)]
#[clap(name = "pd", about = "The Penumbra daemon.", version)]
struct Opt {
    /// Enable Tokio Console support.
    #[clap(long)]
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
        /// If unset, defaults to ~/.penumbra/testnet_data/node0/pd.
        #[clap(long, env = "PENUMBRA_PD_HOME", display_order = 100)]
        home: Option<PathBuf>,
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
        // TODO: Add support for Unix domain sockets, available in tower-abci >=0.10.0
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
        /// The JSON-RPC address of the CometBFT node driving this `pd`
        /// instance.
        ///
        /// This is used to proxy requests from the gRPC server to CometBFT,
        /// so clients only need to connect to one endpoint and don't need to
        /// worry about the peculiarities of CometBFT's JSON-RPC encoding
        /// format.
        #[clap(
            short,
            long,
            env = "PENUMBRA_PD_COMETBFT_PROXY_URL",
            default_value = "http://127.0.0.1:26657",
            display_order = 401,
            // Support old arg name for a while, as we migrate Tendermint -> CometBFT.
            alias = "tendermint-addr",
        )]
        cometbft_addr: Url,

        /// Enable expensive RPCs, such as the trade simulation service.
        /// The trade simulation service allows clients to simulate trades without submitting them.
        /// This is useful for approximating the cost of a trade before submitting it.
        /// But, it is a potential DoS vector, so it is disabled by default.
        #[clap(short, long, display_order = 500)]
        enable_expensive_rpc: bool,
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

    /// Export the storage state the full node.
    Export {
        /// The home directory of the full node.
        #[clap(long, env = "PENUMBRA_PD_HOME", display_order = 100)]
        home: PathBuf,
        /// The directory that the exported state will be written to.
        #[clap(long, display_order = 200)]
        export_path: PathBuf,
        /// Whether to prune the JMT tree.
        #[clap(long, display_order = 300)]
        prune: bool,
    },
    /// Run a migration on the exported storage state of the full node,
    /// and create a genesis file.
    Upgrade {
        /// The directory containing exported state to which the upgrade will be applied.
        #[clap(long, display_order = 200)]
        upgrade_path: PathBuf,
        #[clap(long, display_order = 300)]
        /// Timestamp of the genesis file in RFC3339 format. If unset, defaults to the current time,
        /// unless the migration script overrides it.
        genesis_start: Option<tendermint::time::Time>,
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
        /// The duration, in number of blocks, that a governance proposal
        /// can be voted on.
        #[clap(long)]
        proposal_voting_blocks: Option<u64>,
        /// Base hostname for a validator's p2p service. If multiple validators
        /// exist in the genesis, e.g. via `--validators-input-file`, then
        /// numeric suffixes are automatically added, e.g. "-0", "-1", etc.
        /// Helpful for when you know the validator DNS names ahead of time,
        /// e.g. in Kubernetes service addresses. These option is most useful
        /// to provide peering on a private network setup. If you plan to expose
        /// the validator P2P services to the internet, see the `--external-addresses` option.
        #[clap(long)]
        peer_address_template: Option<String>,

        /// Public addresses and ports for the Tendermint P2P services of the genesis
        /// validator. Accepts comma-separated values, to support multiple validators.
        /// If `--validators-input-file` is used to increase the number
        /// of validators, and the `--external-addresses` flag is set, then the number of
        /// external addresses must equal the number of validators. See the
        /// `--peer-address-template` flag if you don't plan to expose the network
        /// to public peers.
        #[clap(long)]
        // TODO we should support DNS names here. However, there are complications:
        // https://github.com/tendermint/tendermint/issues/1521
        external_addresses: Option<String>,
    },

    /// Like `testnet generate`, but joins the testnet to which the specified node belongs
    Join {
        /// URL of the remote Tendermint RPC endpoint for bootstrapping connection.
        #[clap(
            env = "PENUMBRA_PD_JOIN_URL",
            default_value = "https://rpc.testnet.penumbra.zone"
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
        .with_target(true);
    // The `EnvFilter` layer is used to filter events based on `RUST_LOG`.
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

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
            cometbft_addr,
            enable_expensive_rpc,
        } => {
            tracing::info!(
                ?abci_bind,
                ?grpc_bind,
                ?grpc_auto_https,
                ?metrics_bind,
                %cometbft_addr,
                ?enable_expensive_rpc,
                "starting pd"
            );

            // Ensure we have all necessary parts in the URL
            if !url_has_necessary_parts(&cometbft_addr) {
                anyhow::bail!(
                    "Failed to parse '--cometbft-addr' as URL: {}",
                    cometbft_addr
                )
            }

            // Unpack home directory. Accept an explicit path, but default
            // to a sane value if unspecified.
            let pd_home = match home {
                Some(h) => h,
                None => get_testnet_dir(None).join("node0").join("pd"),
            };
            let rocksdb_home = pd_home.join("rocksdb");

            let storage = Storage::load(rocksdb_home, SUBSTORE_PREFIXES.to_vec())
                .await
                .context("Unable to initialize RocksDB storage")?;

            use penumbra_tower_trace::trace::request_span;

            let consensus = tower::ServiceBuilder::new()
                .layer(request_span::layer(|req: &ConsensusRequest| {
                    req.create_span()
                }))
                .layer(EventIndexLayer::index_all())
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
            let tm_proxy = TendermintProxy::new(cometbft_addr);
            let snapshot = pd::Snapshot {};

            let abci_server = tokio::task::Builder::new()
                .name("abci_server")
                .spawn(
                    tower_abci::v037::Server::builder()
                        .consensus(consensus)
                        .snapshot(snapshot)
                        .mempool(mempool)
                        .info(info.clone())
                        .finish()
                        .context("failed to build abci server")?
                        .listen_tcp(abci_bind),
                )
                .expect("failed to spawn abci server");

            let ibc =
                penumbra_ibc::component::rpc::IbcQuery::new(pd::StorageWrapper(storage.clone()));

            // TODO: Once we migrate to Tonic 0.10.0, we'll be able to use the
            // `Routes` structure to have each component define a method that
            // returns a `Routes` with all of its query services bundled inside.
            //
            // This means we won't have to import all this shit and recite every
            // single service -- we can e.g., have the app crate assemble all of
            // its components' query services into a single `Routes` and then
            // just add that to the gRPC server.

            use cnidarium::rpc::proto::v1alpha1::query_service_server::QueryServiceServer as StorageQueryServiceServer;
            use penumbra_proto::core::{
                app::v1alpha1::query_service_server::QueryServiceServer as AppQueryServiceServer,
                component::{
                    chain::v1alpha1::query_service_server::QueryServiceServer as ChainQueryServiceServer,
                    compact_block::v1alpha1::query_service_server::QueryServiceServer as CompactBlockQueryServiceServer,
                    dex::v1alpha1::query_service_server::QueryServiceServer as DexQueryServiceServer,
                    governance::v1alpha1::query_service_server::QueryServiceServer as GovernanceQueryServiceServer,
                    sct::v1alpha1::query_service_server::QueryServiceServer as SctQueryServiceServer,
                    shielded_pool::v1alpha1::query_service_server::QueryServiceServer as ShieldedPoolQueryServiceServer,
                    stake::v1alpha1::query_service_server::QueryServiceServer as StakeQueryServiceServer,
                },
            };
            use tonic_web::enable as we;

            use cnidarium::rpc::Server as StorageServer;
            use penumbra_app::rpc::Server as AppServer;
            use penumbra_chain::component::rpc::Server as ChainServer;
            use penumbra_compact_block::component::rpc::Server as CompactBlockServer;
            use penumbra_dex::component::rpc::Server as DexServer;
            use penumbra_governance::component::rpc::Server as GovernanceServer;
            use penumbra_sct::component::rpc::Server as SctServer;
            use penumbra_shielded_pool::component::rpc::Server as ShieldedPoolServer;
            use penumbra_stake::component::rpc::Server as StakeServer;

            // Set rather permissive CORS headers for pd's gRPC: the service
            // should be accessible from arbitrary web contexts, such as localhost,
            // or any FQDN that wants to reference its data.
            let cors_layer = CorsLayer::permissive();

            let mut grpc_server = Server::builder()
                .trace_fn(|req| match remote_addr(req) {
                    Some(remote_addr) => {
                        tracing::error_span!("grpc", ?remote_addr)
                    }
                    None => tracing::error_span!("grpc"),
                })
                // Allow HTTP/1, which will be used by grpc-web connections.
                // This is particularly important when running locally, as gRPC
                // typically uses HTTP/2, which requires HTTPS. Accepting HTTP/2
                // allows local applications such as web browsers to talk to pd.
                .accept_http1(true)
                // Add permissive CORS headers, so pd's gRPC services are accessible
                // from arbitrary web contexts, including from localhost.
                .layer(cors_layer)
                // As part of #2932, we are disabling all timeouts until we circle back to our
                // performance story.
                // Sets a timeout for all gRPC requests, but note that in the case of streaming
                // requests, the timeout is only applied to the initial request. This means that
                // this does not prevent long lived streams, for example to allow clients to obtain
                // new blocks.
                // .timeout(std::time::Duration::from_secs(7))
                // Wrap each of the gRPC services in a tonic-web proxy:
                .add_service(we(StorageQueryServiceServer::new(StorageServer::new(
                    storage.clone(),
                ))))
                .add_service(we(AppQueryServiceServer::new(AppServer::new(
                    storage.clone(),
                ))))
                .add_service(we(ChainQueryServiceServer::new(ChainServer::new(
                    storage.clone(),
                ))))
                .add_service(we(CompactBlockQueryServiceServer::new(
                    CompactBlockServer::new(storage.clone()),
                )))
                .add_service(we(DexQueryServiceServer::new(DexServer::new(
                    storage.clone(),
                ))))
                .add_service(we(GovernanceQueryServiceServer::new(
                    GovernanceServer::new(storage.clone()),
                )))
                .add_service(we(SctQueryServiceServer::new(SctServer::new(
                    storage.clone(),
                ))))
                .add_service(we(ShieldedPoolQueryServiceServer::new(
                    ShieldedPoolServer::new(storage.clone()),
                )))
                .add_service(we(StakeQueryServiceServer::new(StakeServer::new(
                    storage.clone(),
                ))))
                .add_service(we(ClientQueryServer::new(ibc.clone())))
                .add_service(we(ChannelQueryServer::new(ibc.clone())))
                .add_service(we(ConnectionQueryServer::new(ibc.clone())))
                .add_service(we(TendermintProxyServiceServer::new(tm_proxy.clone())))
                .add_service(we(tonic_reflection::server::Builder::configure()
                    .register_encoded_file_descriptor_set(penumbra_proto::FILE_DESCRIPTOR_SET)
                    .build()
                    .with_context(|| "could not configure grpc reflection service")?));

            if enable_expensive_rpc {
                grpc_server = grpc_server.add_service(we(SimulationServiceServer::new(
                    DexServer::new(storage.clone()),
                )));
            }

            let grpc_server = if let Some(domain) = grpc_auto_https {
                use pd::auto_https::Wrapper;
                use rustls_acme::{caches::DirCache, AcmeConfig};
                use tokio_stream::wrappers::TcpListenerStream;
                use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

                let mut acme_cache = pd_home.clone();
                acme_cache.push("rustls_acme_cache");

                let grpc_bind = grpc_bind.unwrap_or(
                    "0.0.0.0:443"
                        .parse()
                        .context("failed to parse grpc_bind address")?,
                );
                let bound_listener = TcpListener::bind(grpc_bind)
                    .await
                    .context(format!("Failed to bind HTTPS listener on {}", grpc_bind))?;
                let listener = TcpListenerStream::new(bound_listener);
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
                let grpc_bind = grpc_bind.unwrap_or(
                    "127.0.0.1:8080"
                        .parse()
                        .context("failed to parse grpc_bind address")?,
                );
                tokio::task::Builder::new()
                    .name("grpc_server")
                    .spawn(grpc_server.serve(grpc_bind))
                    .expect("failed to spawn grpc server")
            };

            // Configure a Prometheus recorder and exporter.
            let (recorder, exporter) = PrometheusBuilder::new()
                .with_http_listener(metrics_bind)
                // Set explicit buckets so that Prometheus endpoint emits true histograms, rather
                // than the default distribution type summaries, for time-series data.
                .set_buckets_for_metric(
                    metrics_exporter_prometheus::Matcher::Prefix("penumbra_dex_".to_string()),
                    penumbra_dex::component::metrics::DEX_BUCKETS,
                )?
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
                anyhow::bail!(
                    "output directory {:?} already exists, refusing to overwrite it",
                    output_dir
                );
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
                    peer_address_template,
                    timeout_commit,
                    epoch_duration,
                    unbonding_epochs,
                    active_validator_limit,
                    allocations_input_file,
                    validators_input_file,
                    chain_id,
                    preserve_chain_id,
                    external_addresses,
                    proposal_voting_blocks,
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
                anyhow::bail!(
                    "output directory {:?} already exists, refusing to overwrite it",
                    output_dir
                );
            }

            // Unpack external address information into a vec, since there could be multiple
            // values. We don't yet know how many validators will be in the genesis, but the
            // Testnet::generate constructor will assert that the number of external addresses,
            // if Some, is equal to the number of validators.
            let external_addresses: anyhow::Result<Vec<TendermintAddress>> =
                match external_addresses {
                    Some(a) => a
                        .split(',')
                        .map(|x| {
                            x.parse()
                                .context(format!("Failed to parse external address: {x}"))
                        })
                        .collect(),
                    None => Ok(Vec::new()),
                };

            let external_addresses = external_addresses?;

            // Build and write local configs based on input flags.
            tracing::info!(?chain_id, "Generating network config");
            let t = TestnetConfig::generate(
                &chain_id,
                Some(output_dir),
                peer_address_template,
                Some(external_addresses),
                allocations_input_file,
                validators_input_file,
                timeout_commit,
                active_validator_limit,
                epoch_duration,
                unbonding_epochs,
                proposal_voting_blocks,
            )?;
            tracing::info!(
                n_validators = t.validators.len(),
                chain_id = %t.genesis.chain_id,
                "Writing config files for network"
            );
            t.write_configs()?;
        }
        RootCommand::Export {
            mut home,
            mut export_path,
            prune,
        } => {
            use fs_extra;

            tracing::info!("exporting state to {}", export_path.display());
            let copy_opts = fs_extra::dir::CopyOptions::new();
            home.push("rocksdb");
            let from = [home.as_path()];
            tracing::info!(?home, ?export_path, "copying from data dir to export dir",);
            std::fs::create_dir_all(&export_path)?;
            fs_extra::copy_items(&from, export_path.as_path(), &copy_opts)?;

            tracing::info!("done copying");
            if !prune {
                return Ok(());
            }

            tracing::info!("pruning JMT tree");
            export_path.push("rocksdb");
            let export = Storage::load(export_path, SUBSTORE_PREFIXES.to_vec()).await?;
            let _ = StateDelta::new(export.latest_snapshot());
            // TODO:
            // - add utilities in `cnidarium` to prune a tree
            // - apply the delta to the exported storage
            // - apply checks: root hash, size, etc.
            todo!()
        }
        RootCommand::Upgrade {
            upgrade_path,
            genesis_start,
        } => {
            use upgrade::Upgrade::SimpleUpgrade;
            tracing::info!("upgrading state from {}", upgrade_path.display());
            SimpleUpgrade
                .migrate(upgrade_path.clone(), genesis_start)
                .await
                .context("failed to upgrade state")?;
        }
    }
    Ok(())
}
