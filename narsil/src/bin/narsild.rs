#![allow(clippy::clone_on_copy)]
#![recursion_limit = "512"]
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use console_subscriber::ConsoleLayer;
use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Stack;
use tendermint::abci::{ConsensusRequest, MempoolRequest};

use penumbra_narsil::{ledger::Info, metrics::register_metrics};
use penumbra_proto::narsil::v1alpha1::ledger::ledger_service_server::LedgerServiceServer;

use anyhow::Context;
use clap::{Parser, Subcommand};
use metrics_exporter_prometheus::PrometheusBuilder;
use penumbra_storage::Storage;
use penumbra_tower_trace::remote_addr;
use tokio::runtime;
use tonic::transport::Server;
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Debug, Parser)]
#[clap(
    name = "narsild",
    about = "The narsil daemon.",
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
    /// Start running the narsild ledger service.
    Start {
        /// The path used to store narsild-related data.
        #[clap(long, env = "PENUMBRA_NARSILD_HOME")]
        home: PathBuf,
        /// Bind the ABCI server to this socket.
        #[clap(
            short,
            long,
            env = "PENUMBRA_NARSILD_ABCI_BIND",
            default_value = "127.0.0.1:36658"
        )]
        abci_bind: SocketAddr,
        /// Bind the gRPC server to this socket.
        #[clap(
            short,
            long,
            env = "PENUMBRA_NARSILD_GRPC_BIND",
            default_value = "127.0.0.1:9080"
        )]
        grpc_bind: SocketAddr,
        /// Bind the metrics endpoint to this socket.
        #[clap(
            short,
            long,
            env = "PENUMBRA_NARSILD_METRICS_BIND",
            default_value = "127.0.0.1:9081"
        )]
        metrics_bind: SocketAddr,
    },
}

/// narsild spins up the narsil ledger implementation.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("narsil is currently being forged, please contact the Dwarven-smith Telchar for more details");
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
            grpc_bind,
            metrics_bind,
            abci_bind,
        } => {
            tracing::info!(?abci_bind, ?grpc_bind, ?metrics_bind, "starting narsild");

            let mut rocks_path = home.clone();
            rocks_path.push("rocksdb");

            let storage = Storage::load(rocks_path)
                .await
                .context("Unable to initialize RocksDB storage")?;

            use penumbra_tower_trace::trace::request_span;
            use penumbra_tower_trace::RequestExt;

            let info = Info::new(storage.clone());
            let consensus = tower::ServiceBuilder::new()
                .layer(request_span::layer(|req: &ConsensusRequest| {
                    req.create_span()
                }))
                .service(tower_actor::Actor::new(10, |queue: _| {
                    pd::Consensus::new(storage.clone(), queue).run()
                }));
            let mempool = tower::ServiceBuilder::new()
                .layer(request_span::layer(|req: &MempoolRequest| {
                    req.create_span()
                }))
                .service(tower_actor::Actor::new(10, |queue: _| {
                    pd::Mempool::new(storage.clone(), queue).run()
                }));
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
                        .listen(abci_bind),
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
                        .add_service(tonic_web::enable(LedgerServiceServer::new(info.clone())))
                        .serve(grpc_bind),
                )
                .expect("failed to spawn grpc server");

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

            register_metrics();

            // TODO: better error reporting
            // We error out if a service errors, rather than keep running
            tokio::select! {
                x = abci_server => x?.map_err(|e| anyhow::anyhow!(e))?,
                x = grpc_server => x?.map_err(|e| anyhow::anyhow!(e))?,
            };
        }
    }
    Ok(())
}
