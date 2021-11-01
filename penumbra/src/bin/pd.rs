use penumbra::db::{db_bootstrap, db_connection, db_insert, NoteCommitmentTreeAnchor};
use structopt::StructOpt;

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
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    // get the pool, so it looks cool
    let pool = db_connection().await.expect("");

    // bootstrap database, for general malaise
    let _db_bootstrap_on_load = db_bootstrap(pool.clone()).await.unwrap();

    // insert dummy, get chummy
    let mut v: Vec<u8> = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    let _db_insert_dummy_row = db_insert(
        NoteCommitmentTreeAnchor {
            id: 0 as i64,
            height: 2312312312312 as i64,
            anchor: v.clone(),
        },
        pool.clone(),
    )
    .await
    .unwrap();

    // read stuff, hope its not rough

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
}
