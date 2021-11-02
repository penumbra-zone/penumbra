use structopt::StructOpt;

use penumbra::dbschema::{NoteCommitmentTreeAnchor, PenumbraNoteCommitmentTreeAnchor};
use penumbra::dbutils::{db_bootstrap, db_connection, db_insert, db_read};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pd",
    about = "The Penumbra daemon.", 
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {
    /// Bind the ABCI server to this host.
    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Bind the ABCI server to this port.
    #[structopt(short, long, default_value = "26658")]
    port: u16,

    /// Command to run.
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Generate Genesis state.
    CreateGenesis,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

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

    if opt.cmd.is_none() {
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
        server
            .listen(format!("{}:{}", opt.host, opt.port))
            .await
            .unwrap();
    } else {
        match opt.cmd.unwrap() {
            Command::CreateGenesis => {
                println!("hullo");
            }
        }
    }
}
