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
use futures::stream::TryStreamExt;
use metrics_exporter_prometheus::PrometheusBuilder;
use pd::testnet::{canonicalize_path, generate_tm_config, write_configs, ValidatorKeys};
use penumbra_chain::{genesis::Allocation, params::ChainParameters};
use penumbra_component::stake::{validator::Validator, FundingStream, FundingStreams};
use penumbra_crypto::{keys::SpendKey, DelegationToken, GovernanceKey};
use penumbra_proto::client::v1alpha1::{
    oblivious_query_service_server::ObliviousQueryServiceServer,
    specific_query_service_server::SpecificQueryServiceServer,
};
use penumbra_storage::Storage;
use rand::Rng;
use rand_core::OsRng;
use tokio::{net::TcpListener, runtime};
use tonic::transport::Server;
use tracing_subscriber::{prelude::*, EnvFilter};

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
        /// Bind the metrics endpoint to this port.
        #[clap(short, long, default_value = "9000")]
        metrics_port: u16,
        /// If set, attempt to serve RPC on this domain with auto https
        #[clap(long)]
        grpc_domain: Option<String>,
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
    /// Generates a directory structure containing necessary files to run atestnet based on input
    /// configuration.
    Generate {
        /// Number of blocks per epoch.
        #[clap(long, default_value = "719")]
        epoch_duration: u64,
        /// Number of epochs before unbonding stake is released.
        #[clap(long, default_value = "2")]
        unbonding_epochs: u64,
        /// Maximum number of validators in the consensus set.
        #[clap(long, default_value = "32")]
        active_validator_limit: u64,
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

    /// Like `testnet generate`, but joins the testnet the specified node is part of.
    Join {
        #[clap(default_value = "testnet.penumbra.zone")]
        node: String,
        // Default: node-#
        #[clap(long)]
        moniker: Option<String>,
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
            grpc_domain,
        } => {
            tracing::info!(?host, ?grpc_domain, ?abci_port, ?grpc_port, "starting pd");

            let mut rocks_path = home.clone();
            rocks_path.push("rocksdb");

            let storage = Storage::load(rocks_path)
                .await
                .context("Unable to initialize RocksDB storage")?;

            let consensus = pd::Consensus::new(storage.clone()).await?;
            let mempool = pd::Mempool::new(storage.clone()).await?;
            let info = pd::Info::new(storage.clone());
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
                        .listen(format!("{}:{}", host, abci_port)),
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
                )));

            let grpc_server = if let Some(domain) = grpc_domain {
                use pd::auto_https::Wrapper;
                use rustls_acme::{caches::DirCache, AcmeConfig};
                use tokio_stream::wrappers::TcpListenerStream;
                use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

                let mut acme_cache = home.clone();
                acme_cache.push("rustls_acme_cache");

                let listener =
                    TcpListenerStream::new(TcpListener::bind(format!("{}:{}", host, 443)).await?);
                let tls_incoming = AcmeConfig::new([domain.as_str()])
                    .cache(DirCache::new(acme_cache))
                    .directory_lets_encrypt(true) // Use the production LE environment
                    .incoming(listener.map_ok(|conn| conn.compat()))
                    .map_ok(|incoming| Wrapper {
                        inner: incoming.compat(),
                    });

                tokio::task::Builder::new()
                    .name("grpc_server")
                    .spawn(grpc_server.serve_with_incoming(tls_incoming))
                    .expect("failed to spawn grpc server")
            } else {
                tokio::task::Builder::new()
                    .name("grpc_server")
                    .spawn(
                        grpc_server.serve(
                            format!("{}:{}", host, grpc_port)
                                .parse()
                                .expect("this is a valid address"),
                        ),
                    )
                    .expect("failed to spawn grpc server")
            };

            // Configure a Prometheus recorder and exporter.
            let (recorder, exporter) = PrometheusBuilder::new()
                .with_http_listener(
                    format!("{}:{}", host, metrics_port)
                        .parse::<SocketAddr>()
                        .expect("this is a valid address"),
                )
                .build()
                .expect("failed to build prometheus recorder");

            Stack::new(recorder)
                // Adding the `TracingContextLayer` will add labels from the tracing span to metrics.
                // The only labels to be included are "chain_id" and "role".
                .push(TracingContextLayer::only_allow(&["chain_id", "role"]))
                .install()
                .expect("global recorder already installed");

            // This spawns the HTTP service that lets Prometheus pull metrics from `pd`
            let handle = runtime::Handle::try_current().expect("unable to get runtime handle");
            handle.spawn(exporter);

            pd::register_metrics();

            // TODO: better error reporting
            // We error out if either service errors, rather than keep running
            tokio::select! {
                x = abci_server => x?.map_err(|e| anyhow::anyhow!(e))?,
                x = grpc_server => x?.map_err(|e| anyhow::anyhow!(e))?,
            };
        }

        RootCommand::Testnet {
            tn_cmd: TestnetCommand::UnsafeResetAll {},
            testnet_dir,
        } => {
            // By default output directory will be in `~/.penumbra/testnet_data/`
            let testnet_dir = match testnet_dir {
                Some(o) => o,
                None => canonicalize_path("~/.penumbra/testnet_data"),
            };

            std::fs::remove_dir_all(testnet_dir)?;
        }

        RootCommand::Testnet {
            tn_cmd: TestnetCommand::Join { node, moniker },
            testnet_dir,
        } => {
            // By default output directory will be in `~/.penumbra/testnet_data/`
            let output_dir = match testnet_dir {
                Some(o) => o,
                None => canonicalize_path("~/.penumbra/testnet_data"),
            };

            // If the output directory already exists, bail out, rather than overwriting.
            if output_dir.exists() {
                return Err(anyhow::anyhow!(
                    "output directory {:?} already exists, refusing to overwrite it",
                    output_dir
                ));
            }
            let mut node_dir = output_dir;
            node_dir.push("node0");

            let vk = ValidatorKeys::generate();

            tracing::info!("fetching genesis");
            // We need to download the genesis data and the node ID from the remote node.
            let client = reqwest::Client::new();
            let genesis_json = client
                .get(format!("http://{}:26657/genesis", node))
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?
                .get_mut("result")
                .and_then(|v| v.get_mut("genesis"))
                .ok_or_else(|| anyhow::anyhow!("could not parse JSON from response"))?
                .take();
            let genesis = serde_json::value::from_value(genesis_json)?;
            tracing::info!("fetched genesis");

            let node_id = client
                .get(format!("http://{}:26657/status", node))
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?
                .get_mut("result")
                .and_then(|v| v.get_mut("node_info"))
                .and_then(|v| v.get_mut("id"))
                .ok_or_else(|| anyhow::anyhow!("could not parse JSON from response"))?
                .take();
            tracing::info!(?node_id);
            let node_id = serde_json::value::from_value(node_id)?;
            tracing::info!(?node_id, "fetched node id");

            let node_name = if let Some(moniker) = moniker {
                moniker
            } else {
                format!("node-{}", hex::encode(OsRng.gen::<u32>().to_le_bytes()))
            };
            let tm_config = generate_tm_config(&node_name, &[(node_id, node)]);

            write_configs(node_dir, &vk, &genesis, tm_config)?;
        }

        RootCommand::Testnet {
            tn_cmd:
                TestnetCommand::Generate {
                    // TODO this config is gated on a "populate persistent peers"
                    // setting in the Go tendermint binary. Populating the persistent
                    // peers will be useful in local setups until peer discovery via a seed
                    // works.
                    starting_ip,
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
            use std::{
                fs::File,
                str::FromStr,
                time::{Duration, SystemTime, UNIX_EPOCH},
            };

            use rand::Rng;

            // Build script computes the latest testnet name and sets it as an env variable
            let chain_id = match preserve_chain_id {
                true => chain_id.unwrap_or_else(|| env!("PD_LATEST_TESTNET_NAME").to_string()),
                false => {
                    // If preserve_chain_id is false, we append a random suffix to avoid collisions
                    let randomizer = OsRng.gen::<u32>();
                    let chain_id =
                        chain_id.unwrap_or_else(|| env!("PD_LATEST_TESTNET_NAME").to_string());
                    format!("{}-{}", chain_id, hex::encode(&randomizer.to_le_bytes()))
                }
            };

            use pd::testnet::*;
            use penumbra_chain::genesis;
            use penumbra_crypto::{Address, IdentityKey};
            use tendermint::{node, public_key::Algorithm, Genesis, Time};

            let genesis_time = Time::from_unix_timestamp(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("time travels linearly in a forward direction")
                    .as_secs() as i64,
                0,
            )
            .expect("able to convert current time into Time");

            // By default output directory will be in `~/.penumbra/testnet_data/`
            let output_dir = match testnet_dir {
                Some(o) => o,
                None => canonicalize_path("~/.penumbra/testnet_data"),
            };

            // If the output directory already exists, bail out, rather than overwriting.
            if output_dir.exists() {
                return Err(anyhow::anyhow!(
                    "output directory {:?} already exists, refusing to overwrite it",
                    output_dir
                ));
            }

            // Parse allocations from input file or default to latest testnet allocations computed
            // in the build script
            let mut allocations = if let Some(allocations_input_file) = allocations_input_file {
                let allocations_file = File::open(&allocations_input_file)
                    .with_context(|| format!("cannot open file {:?}", allocations_input_file))?;
                parse_allocations(allocations_file).with_context(|| {
                    format!(
                        "could not parse allocations file {:?}",
                        allocations_input_file
                    )
                })?
            } else {
                static LATEST_ALLOCATIONS: &str =
                    include_str!(env!("PD_LATEST_TESTNET_ALLOCATIONS"));
                parse_allocations(std::io::Cursor::new(LATEST_ALLOCATIONS)).with_context(|| {
                    format!(
                        "could not parse default latest testnet allocations file {:?}",
                        env!("PD_LATEST_TESTNET_ALLOCATIONS")
                    )
                })?
            };

            // Parse validators from input file or default to latest testnet validators computed in
            // the build script
            let testnet_validators = if let Some(validators_input_file) = validators_input_file {
                let validators_file = File::open(&validators_input_file)
                    .with_context(|| format!("cannot open file {:?}", validators_input_file))?;
                parse_validators(validators_file).with_context(|| {
                    format!(
                        "could not parse validators file {:?}",
                        validators_input_file
                    )
                })?
            } else {
                static LATEST_VALIDATORS: &str = include_str!(env!("PD_LATEST_TESTNET_VALIDATORS"));
                parse_validators(std::io::Cursor::new(LATEST_VALIDATORS)).with_context(|| {
                    format!(
                        "could not parse default latest testnet validators file {:?}",
                        env!("PD_LATEST_TESTNET_VALIDATORS")
                    )
                })?
            };

            let mut validator_keys = Vec::<ValidatorKeys>::new();
            // Generate a keypair for each validator
            let num_validator_nodes = testnet_validators.len();
            assert!(
                num_validator_nodes > 0,
                "must have at least one validator node"
            );
            for _ in 0..num_validator_nodes {
                let vk = ValidatorKeys::generate();

                let spend_key = SpendKey::from(vk.validator_spend_key.clone());
                let fvk = spend_key.full_viewing_key();
                let ivk = fvk.incoming();
                let (dest, _dtk_d) = ivk.payment_address(0u64.into());

                // Add a default 1 upenumbra allocation to the validator.
                let identity_key: IdentityKey = IdentityKey(fvk.spend_verification_key().clone());
                let delegation_denom = DelegationToken::from(&identity_key).denom();
                allocations.push(Allocation {
                    address: dest,
                    // Add an initial allocation of 50,000 delegation tokens,
                    // starting them with 50x the individual allocations to discord users.
                    // 50,000 delegation tokens * 1e6 udelegation factor
                    amount: (50_000 * 10u64.pow(6)),
                    denom: delegation_denom.to_string(),
                });

                validator_keys.push(vk);
            }

            let ip_addrs = validator_keys
                .iter()
                .enumerate()
                .map(|(i, _vk)| {
                    let a = starting_ip.octets();
                    Ipv4Addr::new(a[0], a[1], a[2], a[3] + (10 * i as u8))
                })
                .collect::<Vec<_>>();

            let validators = testnet_validators
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let vk = &validator_keys[i];
                    Ok(Validator {
                        // Currently there's no way to set validator keys beyond
                        // manually editing the genesis.json. Otherwise they
                        // will be randomly generated keys.
                        identity_key: IdentityKey(vk.validator_id_vk),
                        governance_key: GovernanceKey(vk.validator_id_vk),
                        consensus_key: vk.validator_cons_pk,
                        name: v.name.clone(),
                        website: v.website.clone(),
                        description: v.description.clone(),
                        enabled: true,
                        funding_streams: FundingStreams::try_from(
                            v.funding_streams
                                .iter()
                                .map(|fs| {
                                    Ok(FundingStream {
                                        address: Address::from_str(&fs.address).map_err(|_| {
                                            anyhow::anyhow!(
                                                "invalid funding stream address in validators.json"
                                            )
                                        })?,
                                        rate_bps: fs.rate_bps,
                                    })
                                })
                                .collect::<Result<Vec<FundingStream>, anyhow::Error>>()?,
                        )
                        .map_err(|_| {
                            anyhow::anyhow!(
                                "unable to construct funding streams from validators.json"
                            )
                        })?,
                        sequence_number: v.sequence_number,
                    })
                })
                .collect::<Result<Vec<Validator>, anyhow::Error>>()?;

            let app_state = genesis::AppState {
                allocations: allocations.clone(),
                chain_params: ChainParameters {
                    chain_id: chain_id.clone(),
                    epoch_duration,
                    unbonding_epochs,
                    active_validator_limit,
                    ..Default::default()
                },
                validators: validators.into_iter().map(Into::into).collect(),
            };

            // Create the genesis data shared by all nodes
            let validator_genesis = Genesis {
                genesis_time,
                chain_id: chain_id
                    .parse::<tendermint::chain::Id>()
                    .expect("able to create chain ID"),
                initial_height: 0,
                consensus_params: tendermint::consensus::Params {
                    block: tendermint::block::Size {
                        max_bytes: 22020096,
                        max_gas: -1,
                        // minimum time increment between consecutive blocks
                        time_iota_ms: 500,
                    },
                    // TODO Should these correspond with values used within `pd` for penumbra epochs?
                    evidence: tendermint::evidence::Params {
                        max_age_num_blocks: 100000,
                        // 1 day
                        max_age_duration: tendermint::evidence::Duration(Duration::new(86400, 0)),
                        max_bytes: 1048576,
                    },
                    validator: tendermint::consensus::params::ValidatorParams {
                        pub_key_types: vec![Algorithm::Ed25519],
                    },
                    version: Some(tendermint::consensus::params::VersionParams { app_version: 0 }),
                },
                // always empty in genesis json
                app_hash: vec![],
                app_state,
                // List of initial validators. Note this may be overridden entirely by
                // the application, and may be left empty to make explicit that the
                // application will initialize the validator set with ResponseInitChain.
                // - https://docs.tendermint.com/v0.32/tendermint-core/using-tendermint.html
                // For penumbra, we can leave this empty since the app_state also contains Validator
                // configs.
                validators: vec![],
            };

            for (n, vk) in validator_keys.iter().enumerate() {
                let node_name = format!("node{}", n);

                // Create the directory for this node
                let mut node_dir = output_dir.clone();
                node_dir.push(node_name.clone());

                // Write this node's config.toml
                // Note that this isn't a re-implementation of the `Config` type from
                // Tendermint (https://github.com/tendermint/tendermint/blob/6291d22f46f4c4f9121375af700dbdafa51577e7/config/config.go#L92)
                // so if they change their defaults or the available fields, that won't be reflected in our template.
                // TODO: grab all peer pubkeys instead of self pubkey
                let my_ip = &ip_addrs[n];
                // Each node should include only the IPs for *other* nodes in their peers list.
                let ips_minus_mine = ip_addrs
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| *p != my_ip)
                    .map(|(n, ip)| {
                        (
                            node::Id::from(validator_keys[n].node_key_pk.ed25519().unwrap()),
                            ip.to_string(),
                        )
                    })
                    .collect::<Vec<_>>();
                let tm_config = generate_tm_config(&node_name, &ips_minus_mine);

                write_configs(node_dir, vk, &validator_genesis, tm_config)?;
            }
        }
    }

    Ok(())
}
