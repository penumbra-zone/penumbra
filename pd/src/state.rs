use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use tendermint::block;
use tokio::sync::watch;
use tracing::instrument;

mod reader;
mod writer;

pub use reader::Reader;
pub use writer::Writer;

#[instrument]
pub async fn new(uri: &str) -> Result<(Reader, Writer)> {
    // Maintain two connection pools, so that reader contention cannot starve
    // the writer.
    #[allow(clippy::zero_prefixed_literal)]
    let (reader_pool, writer_pool) = (
        PgPoolOptions::new()
            .max_connections(16)
            .connect(uri)
            .await?,
        PgPoolOptions::new()
            .max_connections(04)
            .connect(uri)
            .await?,
    );
    // Run migrations prior to building the Reader/Writer so
    // that all of their methods can assume valid db state
    tracing::info!("running migrations");
    sqlx::migrate!("./migrations").run(&writer_pool).await?;
    tracing::info!("finished initializing state");

    // using evmap causes Problems because the read handle isn't Sync,
    // so if a future owns a ReadHandle and also has a borrow of its own data,
    // the borrow isn't Send (even though the reference would never be sent
    // without its referent).
    //let (reader_tmp, writer_tmp) = evmap::new();

    // The watch channel requires setting an initial value.  We'd ideally like
    // to pull default values out of the database, but we haven't created the
    // objects that can do that yet, so we defer that to a Writer::init_caches
    // call below.
    let (chain_params_tx, chain_params_rx) = watch::channel(Default::default());
    let (height_tx, height_rx) = watch::channel(block::Height::from(0u8));
    let (next_rate_data_tx, next_rate_data_rx) = watch::channel(Default::default());
    let (valid_anchors_tx, valid_anchors_rx) = watch::channel(Default::default());

    let reader = Reader {
        pool: reader_pool,
        //tmp: reader_tmp,
        chain_params_rx,
        height_rx,
        next_rate_data_rx,
        valid_anchors_rx,
    };

    // Create a private reader instance for the writer's use
    // using the same connection pool as the writer.
    let mut private_reader = reader.clone();
    private_reader.pool = writer_pool.clone();

    let writer = Writer {
        pool: writer_pool,
        private_reader,
        //tmp: writer_tmp,
        chain_params_tx,
        height_tx,
        next_rate_data_tx,
        valid_anchors_tx,
    };

    writer.init_caches().await?;

    Ok((reader, writer))
}
