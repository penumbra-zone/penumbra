use metrics_exporter_prometheus::PrometheusBuilder;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::fs::File;
use std::net::SocketAddr;
use std::str::FromStr;
use structopt::StructOpt;
use tonic::transport::Server;

use penumbra_crypto::Address;
use penumbra_proto::{
    light_wallet::light_wallet_server::LightWalletServer,
    thin_wallet::thin_wallet_server::ThinWalletServer,
};

use pd::{
    genesis::{generate_genesis_notes, GenesisAddr},
    App, State,
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

    /// Generate Genesis state.
    CreateGenesis {
        /// The chain ID for the new chain
        chain_id: String,
        /// The filename to read genesis data from, either this or `genesis_allocations` must be provided
        #[structopt(short = "f", long = "file", required_unless = "genesis-allocations")]
        file: Option<String>,
        /// The initial set of notes, encoded as a list of tuples "(amount, denom, address)"
        #[structopt(required_unless = "file")]
        genesis_allocations: Vec<GenesisAddr>,
    },
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
                    .add_service(LightWalletServer::new(state.clone()))
                    .serve(
                        format!("{}:{}", host, light_wallet_port)
                            .parse()
                            .expect("this is a valid address"),
                    ),
            );
            let thin_wallet_server = tokio::spawn(
                Server::builder()
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
        Command::CreateGenesis {
            chain_id,
            file,
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

            if !genesis_allocations.is_empty() {
                let genesis_notes = generate_genesis_notes(&mut rng, genesis_allocations);
                let serialized = serde_json::to_string_pretty(&genesis_notes).unwrap();
                println!("\n{}\n", serialized);
                return Ok(());
            }

            if file.is_some() {
                let f = File::open(file.unwrap()).expect("unable to open file");

                // This could be done with Serde but requires adding it to dependencies
                // so this was easier.
                let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(f);
                let mut records = vec![];
                for result in rdr.records() {
                    // The iterator yields Result<StringRecord, Error>, so we check the
                    // error here.
                    let mut record = result?;
                    record.trim();
                    if record.len() != 3 {
                        return Err(anyhow::anyhow!("expected 3-part CSV records"));
                    }

                    let g = GenesisAddr {
                        amount: record[0].parse::<u64>()?,
                        denom: record[1].to_string(),
                        address: Address::from_str(&record[2])?,
                    };
                    records.push(g);
                }

                // let records = io::BufReader::new(f)
                //     .lines()
                //     .map(|x| GenesisAddr::from_str(x?.as_str()))
                //     .collect::<Result<Vec<GenesisAddr>, anyhow::Error>>()?;
                let genesis_notes = generate_genesis_notes(&mut rng, records);
                let serialized = serde_json::to_string_pretty(&genesis_notes).unwrap();
                println!("\n{}\n", serialized);
                return Ok(());
            }
        }
    }

    Ok(())
}
