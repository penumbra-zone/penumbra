use std::pin::Pin;

use anyhow::{Context as _, Result};
use futures::{Stream, StreamExt, TryStreamExt};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tap::{Tap, TapFallible, TapOptional};
use tendermint::abci;
use tracing::{debug, info};

use crate::{opt::Options, AppView, ContextualizedEvent, PgTransaction};

pub struct Indexer {
    opts: Options,
    indexes: Vec<Box<dyn AppView>>,
}

impl Indexer {
    pub fn new(opts: Options) -> Self {
        Self {
            opts,
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

    async fn create_dst_tables(
        pool: &PgPool,
        indexes: &[Box<dyn AppView>],
        app_state: &serde_json::Value,
    ) -> Result<()> {
        let mut dbtx = pool.begin().await?;
        for index in indexes {
            index.init_chain(&mut dbtx, app_state).await?;
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
                    genesis_json,
                },
            indexes,
        } = self;

        // Create a source db, with, for sanity, some read only settings.
        // These will be overrideable by a consumer who knows what they're doing,
        // but prevents basic mistakes.
        // c.f. https://github.com/launchbadge/sqlx/issues/481#issuecomment-727011811
        let src_db = PgPoolOptions::new()
            .after_connect(|conn, _| {
                Box::pin(async move {
                    sqlx::query("SET SESSION CHARACTERISTICS AS TRANSACTION READ ONLY;")
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(&src_database_url)
            .await?;

        let dst_db = PgPool::connect(&dst_database_url).await?;

        // Check if the destination db is initialized
        let dst_db_initialized: bool = sqlx::query_scalar(
            "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_name = 'index_watermark'
        )",
        )
        .fetch_one(&dst_db)
        .await?;

        if !dst_db_initialized {
            tracing::info!("no watermark found, initializing with genesis data");

            // Create the table if it doesn't exist
            sqlx::query("CREATE TABLE index_watermark (events_rowid BIGINT NOT NULL)")
                .execute(&dst_db)
                .await?;

            // Load the genesis JSON to be used populating initial tables
            let genesis_content: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(genesis_json)
                    .context("error reading provided genesis.json file")?,
            )
            .context("error parsing provided genesis.json file")?;
            let app_state = genesis_content
                .get("app_state")
                .ok_or_else(|| anyhow::anyhow!("no app_state key in genesis.json"))?;

            Self::create_dst_tables(&dst_db, &indexes, app_state).await?;
        } else {
            tracing::info!("skipping genesis initialization");
        }

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
        sqlx::query_as::<_, (i64,)>("SELECT MAX(rowid) - $1 FROM events")
            .bind(watermark)
            .fetch_one(src_db)
            .await
            .map(|(count,)| count)?
            .tap(|count| info!(%count, %watermark, "new events since last watermark"));

        let mut scanned_events = 0usize;
        let mut relevant_events = 0usize;

        let mut es = read_events(&src_db, watermark);
        let mut dbtx = dst_db.begin().await?;
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

            for index in indexes {
                if index.is_relevant(&event.as_ref().kind) {
                    tracing::debug!(?event, ?index, "relevant to index");
                    index.index_event(&mut dbtx, &event, &src_db).await?;
                }
            }
            // Mark that we got to at least this event
            update_watermark(&mut dbtx, event.local_rowid).await?;
            // Only commit in batches of <= 1000 events, for about a 5x performance increase when
            // catching up.
            if relevant_events % 1000 == 0 {
                dbtx.commit().await?;
                dbtx = dst_db.begin().await?;
            }
        }
        // Flush out the remaining changes.
        dbtx.commit().await?;

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
        // This query does some shenanigans to ensure good performance.
        // The main trick is that we know that each event has 1 block and <= 1 transaction associated
        // with it, so we can "encourage" (force) Postgres to avoid doing a hash join and
        // then a sort, and instead work from the events in a linear fashion.
        // Basically, this query ends up doing:
        //
        // for event in events >= id:
        //   attach attributes
        //   attach block
        //   attach transaction?
        r#"
SELECT
    events.rowid,
    events.type,
    blocks.height AS block_height,
    tx_results.tx_hash,
    events.attrs
FROM (
    SELECT 
        rowid, 
        type, 
        block_id,
        tx_id,
        jsonb_object_agg(attributes.key, attributes.value) AS attrs
    FROM 
        events 
    LEFT JOIN
        attributes ON rowid = attributes.event_id
    WHERE
        rowid > $1
    GROUP BY 
        rowid, 
        type,
        block_id, 
        tx_id
) events
LEFT JOIN LATERAL (
    SELECT * FROM blocks WHERE blocks.rowid = events.block_id LIMIT 1
) blocks
ON TRUE
LEFT JOIN LATERAL (
    SELECT * FROM tx_results WHERE tx_results.rowid = events.tx_id LIMIT 1
) tx_results
ON TRUE
ORDER BY
    events.rowid ASC
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
