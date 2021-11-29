use metrics::register_counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::net::SocketAddr;
use structopt::StructOpt;
use tonic::transport::Server;

use penumbra_proto::wallet::wallet_server;

use pd::{
    genesis::{generate_genesis_notes, GenesisAddr},
    App, State, WalletApp,
};

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
        /// Bind the wallet service to this port.
        #[structopt(short, long, default_value = "26666")]
        wallet_port: u16,
        /// Bind the metrics endpoint to this port.
        #[structopt(short, long, default_value = "9000")]
        metrics_port: u16,
    },

    /// Generate Genesis state.
    CreateGenesis {
        /// The chain ID for the new chain
        chain_id: String,
        /// The initial set of notes, encoded as a list of tuples "(amount, denom, address)"
        genesis_allocations: Vec<GenesisAddr>,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    match opt.cmd {
        Command::Start {
            host,
            database_uri,
            abci_port,
            wallet_port,
            metrics_port,
        } => {
            tracing::info!(
                ?host,
                ?database_uri,
                ?abci_port,
                ?wallet_port,
                "starting pd"
            );
            // Initialize state
            let state = State::connect(&database_uri).await.unwrap();

            let abci_app = App::new(state.clone()).await.unwrap();
            let wallet_app = WalletApp::new(state);
            let wallet_service_addr = format!("{}:{}", host, wallet_port)
                .parse()
                .expect("this is a valid address");

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

            let wallet_server = tokio::spawn(
                Server::builder()
                    .add_service(wallet_server::WalletServer::new(wallet_app))
                    .serve(wallet_service_addr),
            );

            // This service lets Prometheus pull metrics from `pd`
            let metrics_service_addr: SocketAddr = format!("{}:{}", host, metrics_port)
                .parse()
                .expect("this is a valid address");
            let _metrics_exporter = PrometheusBuilder::new()
                .listen_address(metrics_service_addr)
                .install()
                .expect("metrics service set up");

            // New metrics to track should be added below.
            register_counter!("node_spent_nullifiers_total");

            // TODO: better error reporting
            tokio::select! {
                x = abci_server => {
                    let _ = dbg!(x);
                }
                x = wallet_server => {
                    let _ = dbg!(x);
                }
            };
        }
        Command::CreateGenesis {
            chain_id,
            genesis_allocations,
        } => {
            let chain_id_bytes = chain_id.as_bytes();
            let mut hasher = blake2b_simd::Params::new().hash_length(32).to_state();
            let seed = hasher.update(chain_id_bytes).finalize();

            let mut rng = ChaCha20Rng::from_seed(
                seed.as_bytes()
                    .try_into()
                    .expect("blake2b output is 32 bytes"),
            );

            let genesis_notes = generate_genesis_notes(&mut rng, genesis_allocations);
            let serialized = serde_json::to_string_pretty(&genesis_notes).unwrap();
            println!("\n{}\n", serialized);
        }
    }
}
