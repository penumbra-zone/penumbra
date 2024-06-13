use std::pin::Pin;

use anyhow::Result;
use clap::Parser;
use futures::{Stream, StreamExt, TryStreamExt};
use sqlx::PgPool;
use tap::{Tap, TapFallible, TapOptional};
use tendermint::abci;
use tracing::{debug, info};

use crate::{opt::Options, AppView, ContextualizedEvent, PgTransaction};

pub struct Indexer {
    opts: Options,
    indexes: Vec<Box<dyn AppView>>,
}

impl Indexer {
    pub fn new() -> Self {
        Self {
            opts: Options::parse(),
            indexes: Vec::new(),
        }
    }

    pub fn with_index(mut self, index: impl AppView + 'static) -> Self {
        self.indexes.push(Box::new(index));
        self
    }

    pub fn with_default_tracing(self) -> Self {
        tracing_subscriber::fmt::init();
        self
    }

    async fn create_dst_tables(pool: &PgPool, indexes: &[Box<dyn AppView>]) -> Result<()> {
        let mut dbtx = pool.begin().await?;
        for index in indexes {
            index.init_chain(&mut dbtx).await?;
        }
        dbtx.commit().await?;
        Ok(())
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        tracing::info!(?self.opts);
        let Self {
            opts:
                Options {
                    src_database_url,
                    dst_database_url,
                    chain_id: _,
                    poll_ms,
                },
            indexes,
        } = self;

        let src_db = PgPool::connect(&src_database_url).await?;
        let dst_db = PgPool::connect(&dst_database_url).await?;

        Self::create_dst_tables(&dst_db, &indexes).await?;

        // Create the index_watermark table if it does not exist
        sqlx::query("CREATE TABLE IF NOT EXISTS index_watermark (events_rowid BIGINT NOT NULL)")
            .execute(&dst_db)
            .await?;

        loop {
            Self::tick(&src_db, &dst_db, &indexes).await?;
            tokio::time::sleep(poll_ms).await;
        }
    }

    async fn tick(
        src_db: &PgPool,
        dst_db: &PgPool,
        indexes: &[Box<dyn AppView>],
    ) -> Result<(), anyhow::Error> {
        // Fetch the highest rowid processed so far (the watermark)
        let current_watermark: Option<i64> =
            sqlx::query_as("SELECT events_rowid FROM index_watermark")
                .fetch_optional(dst_db)
                .await?
                .map(|(w,)| w)
                .tap_some(|row_id| debug!(%row_id, "fetched index watermark"))
                .tap_none(|| debug!("no index watermark was present"));

        // Insert initial watermark if not present, so we can use a SET query later
        if current_watermark.is_none() {
            sqlx::query("INSERT INTO index_watermark (events_rowid) VALUES (0)")
                .execute(dst_db)
                .await?
                .tap(|_| debug!("set index watermark to 0"));
        }

        let watermark = current_watermark.unwrap_or(0);

        // Calculate new events count since the last watermark
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM events WHERE rowid > $1")
            .bind(watermark)
            .fetch_one(src_db)
            .await
            .map(|(count,)| count)?
            .tap(|count| info!(%count, %watermark, "new events since last watermark"));

        let mut scanned_events = 0usize;
        let mut relevant_events = 0usize;

        let mut es = read_events(&src_db, watermark);
        while let Some(event) = es.next().await.transpose()? {
            if scanned_events % 1000 == 0 {
                tracing::info!(scanned_events, relevant_events);
            } else {
                tracing::debug!(
                    block_height = %event.block_height,
                    kind = %event.event.kind,
                    scanned_events,
                    relevant_events,
                    "processing event"
                );
            }

            scanned_events += 1;

            // if not relevant then skip making a db tx for the dst db
            if !indexes
                .iter()
                .any(|index| index.is_relevant(&event.as_ref().kind))
            {
                tracing::trace!(kind = %event.as_ref().kind, "event is not relevant to any views");
                continue;
            }

            relevant_events += 1;

            // Otherwise we have something to process. Make a dbtx
            let mut dbtx = dst_db.begin().await?;
            for index in indexes {
                if index.is_relevant(&event.as_ref().kind) {
                    tracing::debug!(?event, ?index, "relevant to index");
                    index.index_event(&mut dbtx, &event).await?;
                }
            }
            // Mark that we got to at least this event
            update_watermark(&mut dbtx, event.local_rowid).await?;
            dbtx.commit().await?;
        }

        Ok(())
    }
}

async fn update_watermark(dbtx: &mut PgTransaction<'_>, watermark: i64) -> Result<()> {
    sqlx::query("UPDATE index_watermark SET events_rowid = $1")
        .bind(watermark)
        .execute(dbtx.as_mut()) // lol, see note on Executor trait about Transaction impl
        .await
        .tap_ok(|affected| {
            debug!(%watermark, "updated index watermark");
            debug_assert_eq!(
                affected.rows_affected(),
                1,
                "only one row should be affected when updating the index watermark"
            );
        })
        .map(|_| ())
        .map_err(anyhow::Error::from)
}

fn read_events(
    src_db: &PgPool,
    watermark: i64,
) -> Pin<Box<dyn Stream<Item = Result<ContextualizedEvent>> + Send + '_>> {
    let event_stream = sqlx::query_as::<_, (i64, String, i64, Option<String>, serde_json::Value)>(
        r#"
SELECT 
    events.rowid, 
    events.type, 
    blocks.height AS block_height,
    tx_results.tx_hash,
    jsonb_object_agg(attributes.key, attributes.value) AS attrs
FROM 
    events 
LEFT JOIN 
    attributes ON events.rowid = attributes.event_id
JOIN 
    blocks ON events.block_id = blocks.rowid
LEFT JOIN 
    tx_results ON events.tx_id = tx_results.rowid
WHERE
    events.rowid > $1
GROUP BY 
    events.rowid, 
    events.type, 
    blocks.height, 
    tx_results.tx_hash
        "#,
    )
    .bind(watermark)
    .fetch(src_db)
    .map_ok(|(local_rowid, type_str, height, tx_hash, attrs)| {
        tracing::debug!(?local_rowid, type_str, height, ?tx_hash);
        let tx_hash: Option<[u8; 32]> = tx_hash.map(|s| {
            hex::decode(s)
                .expect("invalid tx_hash")
                .try_into()
                .expect("expected 32 bytes")
        });
        let block_height = height as u64;

        let serde_json::Value::Object(attrs) = attrs else {
            // saves an allocation below bc we can take ownership
            panic!("expected JSON object");
        };

        let event = abci::Event {
            kind: type_str,
            attributes: attrs
                .into_iter()
                .filter_map(|(k, v)| match v {
                    serde_json::Value::String(s) => Some((k, s)),
                    // we never hit this becasue of how we constructed the query
                    _ => None,
                })
                .map(Into::into)
                .collect(),
        };

        let ce = ContextualizedEvent {
            event,
            block_height,
            tx_hash,
            local_rowid,
        };
        //tracing::info!(?ce);

        ce
    })
    .map_err(|e| anyhow::Error::from(e).context("error reading from database"));

    event_stream.boxed()
}
