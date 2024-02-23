#![allow(clippy::clone_on_copy)]
#![deny(clippy::unwrap_used)]
#![recursion_limit = "512"]
use std::error::Error;
use std::io::IsTerminal as _;

use console_subscriber::ConsoleLayer;
use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Stack;

use anyhow::Context;
use cnidarium::{StateDelta, Storage};
use metrics_exporter_prometheus::PrometheusBuilder;
use pd::{
    cli::{Opt, RootCommand, TestnetCommand},
    migrate::Migration::SimpleMigration,
};
use penumbra_app::SUBSTORE_PREFIXES;

use penumbra_network::config::{get_testnet_dir, url_has_necessary_parts};
use penumbra_network::generate::{PenumbraNetwork, PenumbraNetworkConfig};
use penumbra_network::join::BootstrapNode;

use rand::Rng;
use rand_core::OsRng;
use tendermint_config::net::Address as TendermintAddress;
use tokio::runtime;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Validate options immediately.
    let Opt { tokio_console, cmd } = <Opt as clap::Parser>::parse();

    // Instantiate tracing layers.
    // The MetricsLayer handles enriching metrics output with labels from tracing spans.
    let metrics_layer = MetricsLayer::new();
    // The `FmtLayer` is used to print to the console.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(std::io::stdout().is_terminal())
        .with_target(true);
    // The `EnvFilter` layer is used to filter events based on `RUST_LOG`.
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    // Register the tracing subscribers, conditionally enabling tokio console support
    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(metrics_layer);
    if tokio_console {
        // The ConsoleLayer enables collection of data for `tokio-console`.
        // The `spawn` call will panic if AddrInUse, so we only spawn if enabled.
        let console_layer = ConsoleLayer::builder().with_default_env().spawn();
        registry.with(console_layer).init();
    } else {
        registry.init();
    }

    tracing::info!(?cmd, version = env!("CARGO_PKG_VERSION"), "running command");
    match cmd {
        RootCommand::Start {
            home,
            abci_bind,
            grpc_bind,
            grpc_auto_https,
            acme_staging,
            metrics_bind,
            cometbft_addr,
            enable_expensive_rpc,
        } => {
            // Use the given `grpc_bind` address if one was specified. If not, we will choose a
            // default depending on whether or not `grpc_auto_https` was set. See the
            // `RootCommand::Start::grpc_bind` documentation above.
            let grpc_bind = {
                use std::net::{IpAddr, Ipv4Addr, SocketAddr};
                const HTTP_DEFAULT: SocketAddr =
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
                const HTTPS_DEFAULT: SocketAddr =
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 443);
                let default = || {
                    if grpc_auto_https.is_some() {
                        HTTPS_DEFAULT
                    } else {
                        HTTP_DEFAULT
                    }
                };
                grpc_bind.unwrap_or_else(default)
            };

            // Ensure we have all necessary parts in the URL
            if !url_has_necessary_parts(&cometbft_addr) {
                anyhow::bail!(
                    "Failed to parse '--cometbft-addr' as URL: {}",
                    cometbft_addr
                )
            }

            // Unpack home directory. Accept an explicit path, but default
            // to a sane value if unspecified.
            let pd_home = get_testnet_dir(home).join("node0").join("pd");
            let rocksdb_home = pd_home.join("rocksdb");

            let storage = Storage::load(rocksdb_home, SUBSTORE_PREFIXES.to_vec())
                .await
                .context("Unable to initialize RocksDB storage")?;

            tracing::info!(
                ?abci_bind,
                ?grpc_bind,
                ?grpc_auto_https,
                ?acme_staging,
                ?metrics_bind,
                %cometbft_addr,
                ?enable_expensive_rpc,
                "starting pd"
            );

            let abci_server = tokio::task::Builder::new()
                .name("abci_server")
                .spawn(penumbra_app::server::new(storage.clone()).listen_tcp(abci_bind))
                .expect("failed to spawn abci server");

            let grpc_server =
                penumbra_app::rpc::router(&storage, cometbft_addr, enable_expensive_rpc)?;

            // Create Axum routes for the frontend app.
            let frontend = pd::zipserve::router("/app/", pd::MINIFRONT_ARCHIVE_BYTES);
            let node_status = pd::zipserve::router("/", pd::NODE_STATUS_ARCHIVE_BYTES);

            // Now we drop down a layer of abstraction, from tonic to axum, and merge handlers.
            let router = grpc_server
                .into_router()
                .merge(frontend)
                .merge(node_status)
                // Set rather permissive CORS headers for pd's gRPC: the service
                // should be accessible from arbitrary web contexts, such as localhost,
                // or any FQDN that wants to reference its data.
                .layer(CorsLayer::permissive());

            let make_svc = router.into_make_service();

            // Now start the GRPC server, initializing an ACME client to use as a certificate
            // resolver if auto-https has been enabled.
            macro_rules! spawn_grpc_server {
                ($server:expr) => {
                    tokio::task::Builder::new()
                        .name("grpc_server")
                        .spawn($server.serve(make_svc))
                        .expect("failed to spawn grpc server")
                };
            }
            let grpc_server = axum_server::bind(grpc_bind);
            let grpc_server = match grpc_auto_https {
                Some(domain) => {
                    let (acceptor, acme_worker) =
                        penumbra_auto_https::axum_acceptor(pd_home, domain, !acme_staging);
                    // TODO(kate): we should eventually propagate errors from the ACME worker task.
                    tokio::spawn(acme_worker);
                    spawn_grpc_server!(grpc_server.acceptor(acceptor))
                }
                None => {
                    spawn_grpc_server!(grpc_server)
                }
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
                .map_err(|_| {
                    let msg = format!(
                        "failed to build prometheus recorder; make sure {} is available",
                        &metrics_bind
                    );
                    tracing::error!("{}", msg);
                    anyhow::anyhow!(msg)
                })?;

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

            // We error out if a service errors, rather than keep running.
            // A special attempt is made to detect whether binding to target socket failed;
            // if so, we report that error explicitly, otherwise we fall back to reporting
            // whatever the error was.
            tokio::select! {
                x = abci_server => x?.map_err(|e| {
                    // The display impl on the ABCI error is sufficiently informative,
                    // so we don't need special handling of the failed-to-bind case.
                    let msg = format!("abci server on {} failed: {}", abci_bind, e);
                    tracing::error!("{}", msg);
                    anyhow::anyhow!(msg)
                }
                )?,

                x = grpc_server => x?.map_err(|e| {
                    let mut msg = format!("grpc server on {} failed: {}", grpc_bind, e);
                    // Detect if we have a bind error. We need to unpack nested errors, from
                    // tonic -> hyper -> std. Otherwise, only "transport error" is reported,
                    // which isn't informative enough to take action.
                    if let Some(e) = e.source() {
                        if let Some(e) = e.source() {
                            if let Some(e) = e.downcast_ref::<std::io::Error>() {
                                if e.kind().to_string().contains("address in use") {
                                    msg = format!("grpc bind socket already in use: {}", grpc_bind);
                                }
                            }
                        }
                    }
                    tracing::error!("{}", msg);
                    anyhow::anyhow!(msg)
                }
                )?,
            };
        }

        RootCommand::Testnet {
            tn_cmd: TestnetCommand::UnsafeResetAll {},
            testnet_dir,
        } => {
            let testnet_dir = get_testnet_dir(testnet_dir);
            if testnet_dir.exists() {
                tracing::info!("Removing testnet directory: {}", testnet_dir.display());
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
                    bootstrap_node_url,
                    moniker,
                    external_address,
                    cometbft_rpc_bind,
                    cometbft_p2p_bind,
                },
            testnet_dir,
        } => {
            let output_dir = get_testnet_dir(testnet_dir);

            // If directory exists and is not empty, fail. We allow existing but empty directories
            // to support scripting by external tooling, and also tempdirs in tests.
            if output_dir.exists() && !output_dir.read_dir()?.next().is_none() {
                anyhow::bail!(
                    "output directory {:?} already exists, refusing to overwrite it",
                    output_dir
                );
            }

            // Check whether an external address was set, and parse as TendermintAddress.
            // Required because we accept the CLI opt as a SocketAddr.
            let external_address: Option<TendermintAddress> = match external_address {
                Some(a) => Some(format!("tcp://{}", a).parse()?),
                None => None,
            };

            // Set custom moniker, or default to random string suffix.
            let moniker = match moniker {
                Some(m) => m,
                None => format!("node-{}", hex::encode(OsRng.gen::<u32>().to_le_bytes())),
            };

            let bootstrap_node = BootstrapNode {
                cometbft_url: bootstrap_node_url,
            };

            // Join the target network, looking up network info and writing
            // local configs for pd and cometbft.
            bootstrap_node
                .join(
                    output_dir,
                    &moniker,
                    external_address,
                    cometbft_rpc_bind,
                    cometbft_p2p_bind,
                )
                .await?;
        }

        RootCommand::Testnet {
            tn_cmd:
                TestnetCommand::Generate {
                    peer_address_template,
                    timeout_commit,
                    epoch_duration,
                    unbonding_delay,
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
            let output_dir = get_testnet_dir(testnet_dir);
            // If directory exists and is not empty, fail. We allow existing but empty directories
            // to support scripting by external tooling, and also tempdirs in tests.
            if output_dir.exists() && !output_dir.read_dir()?.next().is_none() {
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

            // Unwrap option
            let chain_id = PenumbraNetwork::generate_chain_id(chain_id, !preserve_chain_id);

            // Build and write local configs based on input flags.
            tracing::info!(?chain_id, "Generating network config");
            let mut net_config = PenumbraNetworkConfig {
                validators_input_file,
                allocations_input_file,
                peer_address_template,
                external_addresses,
                ..Default::default()
            };
            if let Some(t) = timeout_commit {
                net_config.tendermint_timeout_commit = t
            }
            if let Some(l) = active_validator_limit {
                net_config.active_validator_limit = l
            }
            if let Some(u) = unbonding_delay {
                net_config.unbonding_delay = u
            }
            if let Some(p) = proposal_voting_blocks {
                net_config.proposal_voting_blocks = p
            }
            if let Some(e) = epoch_duration {
                net_config.epoch_duration = e
            }

            let mut penumbra_network = PenumbraNetwork::new(Some(chain_id), net_config)?;
            penumbra_network.testnet_dir = output_dir;
            tracing::info!(
                n_validators = penumbra_network.genesis_validators.len(),
                chain_id = %penumbra_network.chain_id,
                "Writing config files for network"
            );
            penumbra_network.write_configs()?;
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
        RootCommand::Migrate {
            target_dir,
            genesis_start,
        } => {
            tracing::info!("migrating state from {}", target_dir.display());
            SimpleMigration
                .migrate(target_dir.clone(), genesis_start)
                .await
                .context("failed to upgrade state")?;
        }
    }
    Ok(())
}
