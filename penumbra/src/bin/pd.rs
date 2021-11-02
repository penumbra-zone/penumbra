use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use structopt::StructOpt;

use penumbra::dbschema::{NoteCommitmentTreeAnchor, PenumbraNoteCommitmentTreeAnchor};
use penumbra::dbutils::{db_bootstrap, db_connection, db_insert, db_read};
use penumbra::genesis::{generate_genesis_notes, GenesisAddr};

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
    /// Start running the ABCI server.
    Start {
        /// Bind the ABCI server to this host.
        #[structopt(short, long, default_value = "127.0.0.1")]
        host: String,
        /// Bind the ABCI server to this port.
        #[structopt(short, long, default_value = "26658")]
        port: u16,
    },

    /// Generate Genesis state.
    CreateGenesis {
        chain_id: String,
        genesis_allocations: Vec<GenesisAddr>,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    match opt.cmd {
        Command::Start { host, port } => {
            // get the pool, cool
            let pool = db_connection().await.expect("");

            // bootstrap database, malaise
            let _db_bootstrap_on_load = db_bootstrap(pool.clone()).await.unwrap();

            // insert dummy, chummy
            let v: Vec<u8> = vec![6; 32];
            let _db_insert_dummy_row = db_insert(
                PenumbraNoteCommitmentTreeAnchor::from(NoteCommitmentTreeAnchor {
                    id: 0,
                    height: 1337 as i64,
                    anchor: v,
                }),
                pool.clone(),
            )
            .await
            .unwrap();

            // read stuff, rough
            let _db_read_dummy_row = db_read(pool.clone()).await.unwrap();
            println!(
                "raw height {} raw anchor {:?}",
                _db_read_dummy_row[0].height, _db_read_dummy_row[0].anchor
            );

            // app
            let app = penumbra::App::default();

            use tower_abci::{split, Server};

            let (consensus, mempool, snapshot, info) = split::service(app, 1);

            let server = Server::builder()
                .consensus(consensus)
                .snapshot(snapshot)
                .mempool(mempool)
                .info(info)
                .finish()
                .unwrap();

            // Run the ABCI server.
            server.listen(format!("{}:{}", host, port)).await.unwrap();
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
