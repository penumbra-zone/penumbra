use std::{collections::HashMap, pin::Pin, sync::Arc};

use futures::{Stream, StreamExt, TryStreamExt};
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Transaction};
use tendermint::abci;

use crate::{
    index::{BlockEvents, EventBatch},
    ContextualizedEvent,
};

/// Create a Database, with, for sanity, some read only settings.
///
/// These will be overrideable by a consumer who knows what they're doing,
/// but prevents basic mistakes.
/// c.f. https://github.com/launchbadge/sqlx/issues/481#issuecomment-727011811
async fn read_only_db(url: &str) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET SESSION CHARACTERISTICS AS TRANSACTION READ ONLY;")
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect(url)
        .await
        .map_err(Into::into)
}

async fn read_write_db(url: &str) -> anyhow::Result<PgPool> {
    PgPoolOptions::new().connect(url).await.map_err(Into::into)
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Height(u64);

impl Height {
    pub fn post_genesis() -> Self {
        Height(1)
    }
    /// Return the last height in the batch, and then the first height in the next batch.
    pub fn advance(self, batch_size: u64, max_height: Height) -> (Height, Height) {
        let last = Height::from(self.0 + batch_size - 1).min(max_height);
        let next_first = Height::from(last.0 + 1);
        (last, next_first)
    }

    pub fn next(self) -> Height {
        Self(self.0 + 1)
    }
}

impl From<u64> for Height {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl TryFrom<i64> for Height {
    type Error = anyhow::Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self(u64::try_from(value)?))
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for Height {
    fn decode(
        value: <Postgres as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(Height::try_from(
            <i64 as sqlx::Decode<'r, Postgres>>::decode(value)?,
        )?)
    }
}

impl sqlx::Type<Postgres> for Height {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <i64 as sqlx::Type<Postgres>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, Postgres> for Height {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        <i64 as sqlx::Encode<'q, Postgres>>::encode(
            i64::try_from(self.0).expect("height should never exceed i64::MAX"),
            buf,
        )
    }
}

#[derive(Debug, Clone)]
pub struct IndexingState {
    src: PgPool,
    dst: PgPool,
}

impl IndexingState {
    async fn create_watermark_table(&self) -> anyhow::Result<()> {
        sqlx::query(
            "
        CREATE TABLE IF NOT EXISTS index_watermarks (
            index_name TEXT PRIMARY KEY,
            height BIGINT NOT NULL
        )
        ",
        )
        .execute(&self.dst)
        .await?;
        Ok(())
    }

    /// The largest height for which we know we have all events.
    pub async fn src_height(&self) -> anyhow::Result<Height> {
        // We may be currently indexing events for this block.
        let res: Option<Height> = sqlx::query_scalar("SELECT MAX(height) - 1 FROM blocks")
            .fetch_optional(&self.src)
            .await?;
        Ok(res.unwrap_or_default())
    }

    pub async fn index_heights(&self) -> anyhow::Result<HashMap<String, Height>> {
        let rows: Vec<(String, Height)> =
            sqlx::query_as("SELECT index_name, height FROM index_watermarks")
                .fetch_all(&self.dst)
                .await?;
        Ok(rows.into_iter().collect())
    }

    pub async fn update_index_height(
        dbtx: &mut sqlx::Transaction<'_, Postgres>,
        name: &str,
        height: Height,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "
        INSERT INTO index_watermarks 
        VALUES ($1, $2) 
        ON CONFLICT (index_name) 
        DO UPDATE SET height = excluded.height
        ",
        )
        .bind(name)
        .bind(height)
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    pub async fn event_batch(&self, first: Height, last: Height) -> anyhow::Result<EventBatch> {
        // The amount of events we expect a block to have.
        const WORKING_CAPACITY: usize = 32;

        let mut by_height = Vec::with_capacity((last.0 - first.0 + 1) as usize);
        let mut event_stream =
            sqlx::query_as::<_, (i64, String, i64, Option<String>, serde_json::Value)>(
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
    events.height,
    tx_results.tx_hash,
    events.attrs
FROM (
    SELECT 
        (SELECT height FROM blocks WHERE blocks.rowid = block_id) as height,
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
        block_id >= (SELECT rowid FROM blocks where height = $1)
    AND
        block_id <= (SELECT rowid FROM blocks where height = $2)
    GROUP BY 
        rowid, 
        type,
        block_id, 
        tx_id
    ORDER BY
        rowid ASC
) events
LEFT JOIN LATERAL (
    SELECT * FROM tx_results WHERE tx_results.rowid = events.tx_id LIMIT 1
) tx_results
ON TRUE
ORDER BY
    events.rowid ASC
        "#,
            )
            .bind(first)
            .bind(last)
            .fetch(&self.src)
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

                ce
            })
            .map_err(|e| anyhow::Error::from(e).context("error reading from database"));

        let mut height = first.0;
        let mut current_batch = BlockEvents {
            height: first.0,
            events: Vec::with_capacity(WORKING_CAPACITY),
        };
        while let Some(e) = event_stream.try_next().await? {
            assert!(e.block_height >= height);
            if e.block_height > height {
                by_height.push(current_batch);
                current_batch = BlockEvents {
                    height,
                    events: Vec::with_capacity(WORKING_CAPACITY),
                };
                height = e.block_height;
            }
            current_batch.events.push(e);
        }
        // Flush the current block, and create empty ones for the remaining heights.
        while height <= last.0 {
            by_height.push(current_batch);
            current_batch = BlockEvents {
                height,
                events: Vec::new(),
            };
            height += 1;
        }
        Ok(EventBatch {
            first_height: first.0,
            last_height: last.0,
            by_height: Arc::new(by_height),
        })
    }

    pub async fn init(src_url: &str, dst_url: &str) -> anyhow::Result<Self> {
        tracing::info!(url = src_url, "connecting to raw database");
        tracing::info!(url = dst_url, "connecting to derived database");
        let (src, dst) = tokio::try_join!(read_only_db(src_url), read_write_db(dst_url))?;
        let out = Self { src, dst };
        out.create_watermark_table().await?;
        Ok(out)
    }

    pub async fn begin_transaction(&self) -> anyhow::Result<Transaction<'_, Postgres>> {
        Ok(self.dst.begin().await?)
    }
}
