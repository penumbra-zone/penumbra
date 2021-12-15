use std::net::SocketAddr;

use metrics_exporter_prometheus::PrometheusBuilder;
use pd::{genesis, App, State};
use penumbra_proto::{
    light_wallet::light_wallet_server::LightWalletServer,
    thin_wallet::thin_wallet_server::ThinWalletServer,
};
use penumbra_stake::{FundingStream, Validator};
use rand_core::OsRng;
use structopt::StructOpt;
use tonic::transport::Server;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pd",
    about = "The Penumbra daemon.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {
    /// Command to run.
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Start running the ABCI and wallet services.
    Start {
        /// The URI used to connect to the Postgres database.
        #[structopt(short, long)]
        database_uri: String,
        /// Bind the services to this host.
        #[structopt(short, long, default_value = "127.0.0.1")]
        host: String,
        /// Bind the ABCI server to this port.
        #[structopt(short, long, default_value = "26658")]
        abci_port: u16,
        /// Bind the light wallet service to this port.
        #[structopt(short, long, default_value = "26666")]
        light_wallet_port: u16,
        /// Bind the thin wallet service to this port.
        #[structopt(short, long, default_value = "26667")]
        thin_wallet_port: u16,
        /// Bind the metrics endpoint to this port.
        #[structopt(short, long, default_value = "9000")]
        metrics_port: u16,
    },

    /// Prints a sample `app_data` JSON object that can act as a template for
    /// editing genesis configuration.
    CreateGenesisTemplate,
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
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    match opt.cmd {
        Command::Start {
            host,
            database_uri,
            abci_port,
            light_wallet_port,
            thin_wallet_port,
            metrics_port,
        } => {
            tracing::info!(
                ?host,
                ?database_uri,
                ?abci_port,
                ?light_wallet_port,
                ?thin_wallet_port,
                "starting pd"
            );
            // Initialize state
            let state = State::connect(&database_uri).await.unwrap();

            let abci_app = App::new(state.clone()).await.unwrap();

            let (consensus, mempool, snapshot, info) = tower_abci::split::service(abci_app, 1);

            let abci_server = tokio::spawn(
                tower_abci::Server::builder()
                    .consensus(consensus)
                    .snapshot(snapshot)
                    .mempool(mempool)
                    .info(info)
                    .finish()
                    .unwrap()
                    .listen(format!("{}:{}", host, abci_port)),
            );

            let light_wallet_server = tokio::spawn(
                Server::builder()
                    .trace_fn(|req| match remote_addr(req) {
                        Some(remote_addr) => tracing::error_span!("light_wallet", ?remote_addr),
                        None => tracing::error_span!("light_wallet"),
                    })
                    .add_service(LightWalletServer::new(state.clone()))
                    .serve(
                        format!("{}:{}", host, light_wallet_port)
                            .parse()
                            .expect("this is a valid address"),
                    ),
            );
            let thin_wallet_server = tokio::spawn(
                Server::builder()
                    .trace_fn(|req| match remote_addr(req) {
                        Some(remote_addr) => tracing::error_span!("thin_wallet", ?remote_addr),
                        None => tracing::error_span!("thin_wallet"),
                    })
                    .add_service(ThinWalletServer::new(state.clone()))
                    .serve(
                        format!("{}:{}", host, thin_wallet_port)
                            .parse()
                            .expect("this is a valid address"),
                    ),
            );

            // This service lets Prometheus pull metrics from `pd`
            PrometheusBuilder::new()
                .listen_address(
                    format!("{}:{}", host, metrics_port)
                        .parse::<SocketAddr>()
                        .expect("this is a valid address"),
                )
                .install()
                .expect("metrics service set up");

            pd::register_all_metrics();

            // TODO: better error reporting
            // We error out if either service errors, rather than keep running
            tokio::select! {
                x = abci_server => x?.map_err(|e| anyhow::anyhow!(e))?,
                x = light_wallet_server => x?.map_err(|e| anyhow::anyhow!(e))?,
                x = thin_wallet_server => x?.map_err(|e| anyhow::anyhow!(e))?,
            };
        }
        Command::CreateGenesisTemplate => {
            use penumbra_crypto::keys::SpendKey;

            // Use this to make up some addresses
            let sk = SpendKey::generate(OsRng);
            let ivk = sk.incoming_viewing_key();

            let validator_sk =
                tendermint::PrivateKey::Ed25519(ed25519_consensus::SigningKey::new(OsRng));
            let validator_pk = validator_sk.public_key();

            let app_state = genesis::AppState {
                allocations: vec![
                    genesis::Allocation {
                        amount: 1_000_000,
                        denom: "upenumbra".to_string(),
                        address: ivk.payment_address(10u8.into()).0,
                    },
                    genesis::Allocation {
                        amount: 10_000,
                        denom: "gm".to_string(),
                        address: ivk.payment_address(11u8.into()).0,
                    },
                    genesis::Allocation {
                        amount: 1_000,
                        denom: "cubes".to_string(),
                        address: ivk.payment_address(12u8.into()).0,
                    },
                ],
                // Set a shorter epoch duration here for testing purposes and to
                // try to avoid baking in assumptions about the epoch length
                epoch_duration: 300,
                validators: vec![Validator::new(
                    validator_pk,
                    100u32.into(),
                    vec![FundingStream {
                        address: ivk.payment_address(0u8.into()).0,
                        rate_bps: 200,
                    }],
                )],
            };

            println!("// Edit the following template according to your needs");
            println!("\n{}\n", serde_json::to_string_pretty(&app_state)?);
        }
    }

    Ok(())
}
