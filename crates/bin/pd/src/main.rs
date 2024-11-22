#![allow(clippy::clone_on_copy)]
#![deny(clippy::unwrap_used)]
#![recursion_limit = "512"]
use std::io::IsTerminal as _;
use std::{error::Error, process::exit};

use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Stack;

use anyhow::{anyhow, Context};
use cnidarium::Storage;
use metrics_exporter_prometheus::PrometheusBuilder;
use pd::{
    cli::{NetworkCommand, Opt, RootCommand},
    migrate::Migration::{Mainnet2, ReadyToStart},
    network::{
        config::{get_network_dir, parse_tm_address, url_has_necessary_parts},
        generate::NetworkConfig,
        join::network_join,
    },
};
use penumbra_app::app_version::assert_latest_app_version;
use penumbra_app::SUBSTORE_PREFIXES;
use rand::Rng;
use rand_core::OsRng;
use tendermint_config::net::Address as TendermintAddress;
use tower_http::cors::CorsLayer;
use tracing::Instrument as _;
use tracing_subscriber::{prelude::*, EnvFilter};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Validate options immediately.
    let Opt { cmd } = <Opt as clap::Parser>::parse();

    // Instantiate tracing layers.
    // The MetricsLayer handles enriching metrics output with labels from tracing spans.
    let metrics_layer = MetricsLayer::new();
    // The `FmtLayer` is used to print to the console.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(std::io::stdout().is_terminal())
        .with_target(true);
    // The `EnvFilter` layer is used to filter events based on `RUST_LOG`.
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    // Register the tracing subscribers.
    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(metrics_layer);
    registry.init();

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
            let pd_home = match home {
                Some(h) => h,
                None => get_network_dir(None).join("node0").join("pd"),
            };
            let rocksdb_home = pd_home.join("rocksdb");

            let storage = Storage::load(rocksdb_home, SUBSTORE_PREFIXES.to_vec())
                .await
                .context(
                    "Unable to initialize RocksDB storage - is there another `pd` process running?",
                )?;
            assert_latest_app_version(storage.clone()).await?;

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

            if penumbra_app::app::App::is_ready(storage.latest_snapshot()).await {
                tracing::info!("application ready to start");
            } else {
                tracing::warn!("application is halted, refusing to start");
                exit(0)
            }

            let abci_server = tokio::task::spawn(
                penumbra_app::server::new(storage.clone()).listen_tcp(abci_bind),
            );

            let tm_proxy = penumbra_tendermint_proxy::TendermintProxy::new(cometbft_addr);
            let grpc_server = penumbra_app::rpc::router(&storage, tm_proxy, enable_expensive_rpc)?;

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
            // resolver if auto-https has been enabled. if auto-https is not enabled, we will
            // instead spawn a future that will never return.
            let grpc_server = axum_server::bind(grpc_bind);
            let (grpc_server, acme_worker) = match grpc_auto_https {
                Some(domain) => {
                    let (acceptor, acme_worker) =
                        penumbra_auto_https::axum_acceptor(pd_home, domain, !acme_staging);
                    let acme_worker = tokio::spawn(acme_worker);
                    let grpc_server =
                        tokio::task::spawn(grpc_server.acceptor(acceptor).serve(make_svc));
                    (grpc_server, acme_worker)
                }
                None => {
                    let acme_worker = tokio::task::spawn(futures::future::pending());
                    let grpc_server = tokio::task::spawn(grpc_server.serve(make_svc));
                    (grpc_server, acme_worker)
                }
            };

            // Configure a Prometheus recorder and exporter.
            use penumbra_dex::component::metrics::PrometheusBuilderExt;
            let (recorder, exporter) = PrometheusBuilder::new()
                .with_http_listener(metrics_bind)
                // Set explicit buckets so that Prometheus endpoint emits true histograms, rather
                // than the default distribution type summaries, for time-series data.
                .set_buckets_for_dex_metrics()?
                .build()
                .map_err(|e| {
                    let msg = format!(
                        "failed to build prometheus recorder; make sure {} is available",
                        &metrics_bind
                    );
                    tracing::error!(?e, ?msg);
                    anyhow!(msg)
                })?;

            Stack::new(recorder)
                // Adding the `TracingContextLayer` will add labels from the tracing span to metrics.
                // The only labels to be included are "chain_id" and "role".
                .push(TracingContextLayer::only_allow(["chain_id", "role"]))
                .install()
                .expect("global recorder already installed");

            // Spawn the HTTP service that lets Prometheus pull metrics from `pd`, and then
            // register pd's metrics with the exporter.
            tokio::spawn(exporter);
            pd::register_metrics();
            tokio::spawn(pd::metrics::sleep_worker::run());
            tokio::spawn(pd::metrics::cpu_worker::run());

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

                // if the acme worker returns an error, let's propagate it!
                x = acme_worker => x?.map_err(|error| {
                    let msg = format!("acme worker failed: {error}");
                    tracing::error!("{}", msg);
                    anyhow::anyhow!(msg)
                })?,
            };
        }

        RootCommand::Network {
            net_cmd: NetworkCommand::UnsafeResetAll {},
            network_dir,
        } => {
            let network_dir = get_network_dir(network_dir);
            if network_dir.exists() {
                tracing::info!("removing network directory: {}", network_dir.display());
                std::fs::remove_dir_all(network_dir)?;
            } else {
                tracing::info!(
                    "network directory does not exist, so not removing: {}",
                    network_dir.display()
                );
            }
        }

        RootCommand::Network {
            net_cmd:
                NetworkCommand::Join {
                    node,
                    archive_url,
                    moniker,
                    external_address,
                    tendermint_rpc_bind,
                    tendermint_p2p_bind,
                    leave_archive,
                },
            network_dir,
        } => {
            let output_dir = get_network_dir(network_dir);

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

            // Join the target network, looking up network info and writing
            // local configs for pd and cometbft.
            network_join(
                output_dir.clone(),
                node,
                &node_name,
                external_address,
                tendermint_rpc_bind,
                tendermint_p2p_bind,
            )
            .await?;

            // Download and extract archive URL, if set.
            if let Some(archive_url) = archive_url {
                pd::network::join::unpack_state_archive(archive_url, output_dir, leave_archive)
                    .await?;
            }
        }

        RootCommand::Network {
            net_cmd:
                NetworkCommand::Generate {
                    peer_address_template,
                    timeout_commit,
                    epoch_duration,
                    unbonding_delay,
                    active_validator_limit,
                    allocations_input_file,
                    allocation_address,
                    validators_input_file,
                    chain_id,
                    gas_price_simple,
                    preserve_chain_id,
                    external_addresses,
                    proposal_voting_blocks,
                },
            network_dir,
        } => {
            // Build script computes the latest testnet name and sets it as an env variable
            let chain_id = match preserve_chain_id {
                true => chain_id.unwrap_or_else(|| env!("PD_LATEST_TESTNET_NAME").to_string()),
                false => {
                    // If preserve_chain_id is false, we append a random suffix to avoid collisions
                    let randomizer = OsRng.gen::<u32>();
                    let chain_id =
                        chain_id.unwrap_or_else(|| env!("PD_LATEST_TESTNET_NAME").to_string());
                    // We insert an 'x' in the randomized hex string to ensure it's not parsed as a
                    // revision id.
                    format!("{}-x{}", chain_id, hex::encode(randomizer.to_le_bytes()))
                }
            };

            let output_dir = get_network_dir(network_dir);
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
            let t = NetworkConfig::generate(
                &chain_id,
                Some(output_dir),
                peer_address_template,
                Some(external_addresses),
                allocations_input_file,
                allocation_address,
                validators_input_file,
                timeout_commit,
                active_validator_limit,
                epoch_duration,
                unbonding_delay,
                proposal_voting_blocks,
                gas_price_simple,
            )?;
            tracing::info!(
                n_validators = t.validators.len(),
                chain_id = %t.genesis.chain_id,
                "Writing config files for network"
            );
            t.write_configs()?;
        }
        RootCommand::Export {
            home,
            export_directory,
            export_archive,
            prune,
        } => {
            use fs_extra;

            // Export state as directory.
            let src_rocksdb_dir = home.join("rocksdb");
            tracing::info!(
                "copying node state {} -> {}",
                src_rocksdb_dir.display(),
                export_directory.display()
            );
            std::fs::create_dir_all(&export_directory)?;
            let copy_opts = fs_extra::dir::CopyOptions::new();
            fs_extra::copy_items(
                &[src_rocksdb_dir.as_path()],
                export_directory.as_path(),
                &copy_opts,
            )?;
            tracing::info!("finished copying node state");

            let dst_rocksdb_dir = export_directory.join("rocksdb");
            // If prune=true, then export-directory is required, because we must munge state prior
            // to compressing. So we'll just mandate the presence of the --export-directory arg
            // always.
            if prune {
                unimplemented!("storage pruning is unimplemented (for now)")
            }

            // Compress to tarball if requested.
            if let Some(archive_filepath) = export_archive {
                pd::migrate::archive_directory(
                    dst_rocksdb_dir.clone(),
                    archive_filepath.clone(),
                    Some("rocksdb".to_owned()),
                )?;
                tracing::info!("export complete: {}", archive_filepath.display());
            } else {
                // Provide friendly "OK" message that's still accurate without archiving.
                tracing::info!("export complete: {}", export_directory.display());
            }
        }
        RootCommand::Migrate {
            home,
            comet_home,
            force,
            ready_to_start,
        } => {
            let (pd_home, comet_home) = match home {
                Some(h) => (h, comet_home),
                None => {
                    // If no pd_home was configured, we're assuming we set up the
                    // data in the default location, in which case we also know where comet lives.
                    let base = get_network_dir(None).join("node0");
                    (base.join("pd"), Some(base.join("cometbft")))
                }
            };
            let pd_migrate_span = tracing::error_span!("pd_migrate");
            pd_migrate_span
                .in_scope(|| tracing::info!("migrating pd state in {}", pd_home.display()));

            if ready_to_start {
                tracing::info!("disabling halt order in local state");
                ReadyToStart
                    .migrate(pd_home, comet_home, None, force)
                    .instrument(pd_migrate_span)
                    .await
                    .context("failed to disable halt bit in local state")?;
                exit(0)
            }

            let genesis_start = pd::migrate::last_block_timestamp(pd_home.clone()).await?;
            tracing::info!(?genesis_start, "last block timestamp");
            Mainnet2
                .migrate(pd_home.clone(), comet_home, Some(genesis_start), force)
                .instrument(pd_migrate_span)
                .await
                .context("failed to upgrade state")?;
        }
    }
    Ok(())
}
