use anyhow::anyhow;
use futures::{Stream, StreamExt, TryStreamExt};
use prost::Message as _;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
use tendermint::abci::{self, Event};

use crate::database::{read_only_db, read_write_db};
use crate::index::{BlockEvents, EventBatch, Version};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, sqlx::Type)]
#[sqlx(transparent)]
pub struct Height(i64);

impl Height {
    pub fn post_genesis() -> Self {
        Self(1)
    }

    /// Return the last height in the batch, and then the first height in the next batch.
    pub fn advance(self, batch_size: u64, max_height: Self) -> (Self, Self) {
        let last = Self::from(self.0 as u64 + batch_size - 1).min(max_height);
        let next_first = Self(last.0 + 1);
        (last, next_first)
    }

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<u64> for Height {
    fn from(value: u64) -> Self {
        Self(value.try_into().unwrap_or(i64::MAX))
    }
}

impl From<Height> for u64 {
    fn from(value: Height) -> Self {
        value.0.try_into().unwrap_or_default()
    }
}

/// The state of a particular index.
#[derive(Default, Debug, Clone, Copy)]
pub struct IndexState {
    /// What version this particular index has been using.
    pub version: Version,
    /// What height this particular index has reached.
    pub height: Height,
}

#[derive(Debug, Clone)]
pub struct IndexingManager {
    src: PgPool,
    dst: PgPool,
}

impl IndexingManager {
    async fn create_watermark_table(&self) -> anyhow::Result<()> {
        sqlx::query(
            "
        CREATE TABLE IF NOT EXISTS index_watermarks (
            index_name TEXT PRIMARY KEY,
            height BIGINT NOT NULL,
            version BIGINT
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

    pub async fn index_state(&self, name: &str) -> anyhow::Result<IndexState> {
        let row: Option<(Height, Version)> =
            sqlx::query_as("SELECT height, version FROM index_watermarks WHERE index_name = $1")
                .bind(name)
                .fetch_optional(&self.dst)
                .await?;
        Ok(row
            .map(|(height, version)| IndexState { height, version })
            .unwrap_or_default())
    }

    pub async fn index_states(&self) -> anyhow::Result<HashMap<String, IndexState>> {
        let rows: Vec<(String, Height, Version)> =
            sqlx::query_as("SELECT index_name, height, version FROM index_watermarks")
                .fetch_all(&self.dst)
                .await?;
        Ok(rows
            .into_iter()
            .map(|(name, height, version)| (name, IndexState { height, version }))
            .collect())
    }

    pub async fn update_index_state(
        dbtx: &mut sqlx::Transaction<'_, Postgres>,
        name: &str,
        new_state: IndexState,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "
        INSERT INTO index_watermarks 
        VALUES ($1, $2, $3) 
        ON CONFLICT (index_name) 
        DO UPDATE SET
            height = excluded.height,
            version = excluded.version
        ",
        )
        .bind(name)
        .bind(new_state.height)
        .bind(new_state.version)
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    fn transactions_between(
        &self,
        first: Height,
        last: Height,
    ) -> impl Stream<Item = anyhow::Result<(Height, [u8; 32], Vec<u8>)>> + '_ {
        async fn parse_row(
            row: (i64, String, Vec<u8>),
        ) -> anyhow::Result<(Height, [u8; 32], Vec<u8>)> {
            let tx_hash: [u8; 32] = hex::decode(row.1)?
                .try_into()
                .map_err(|_| anyhow!("expected 32 byte hash"))?;
            let tx_result = tendermint_proto::v0_37::abci::TxResult::decode(row.2.as_slice())?;
            let transaction = tx_result.tx.to_vec();
            let height = Height(row.0);
            Ok((height, tx_hash, transaction))
        }

        sqlx::query_as::<_, (i64, String, Vec<u8>)>(
            r#"
SELECT height, tx_hash, tx_result
FROM blocks
JOIN tx_results ON blocks.rowid = tx_results.block_id
WHERE
    height >= $1
AND
    height <= $2
"#,
        )
        .bind(first)
        .bind(last)
        .fetch(&self.src)
        .map_err(|e| anyhow::Error::from(e).context("error reading from database"))
        .and_then(parse_row)
    }

    fn events_between(
        &self,
        first: Height,
        last: Height,
    ) -> impl Stream<Item = anyhow::Result<(Height, Event, Option<[u8; 32]>, i64)>> + '_ {
        sqlx::query_as::<_, (i64, String, Height, Option<String>, serde_json::Value)>(
            // This query does some shenanigans to ensure good performance.
            // The main trick is that we know that each event has 1 block and <= 1 transaction associated
            // with it, so we can "encourage" (force) Postgres to avoid doing a hash join and
            // then a sort, and instead work from the events in a linear fashion.
            // Basically, this query ends up doing:
            //
            // for event in events >= id:
            //   attach attributes
            //   attach block
            //   attach transaction hash?
            r#"
WITH blocks AS (
  SELECT * FROM blocks WHERE height >= $1 AND height <= $2
),
filtered_events AS (
  SELECT e.block_id, e.rowid, e.type, e.tx_id, b.height
  FROM events e
  JOIN blocks b ON e.block_id = b.rowid
),
events_with_attrs AS (
  SELECT
      f.block_id,
      f.rowid,
      f.type,
      f.tx_id,
      f.height,
      jsonb_object_agg(a.key, a.value) AS attrs
  FROM filtered_events f
  LEFT JOIN attributes a ON f.rowid = a.event_id
  GROUP BY f.block_id, f.rowid, f.type, f.tx_id, f.height
)
SELECT e.rowid, e.type, e.height, tx.tx_hash, e.attrs
FROM events_with_attrs e
LEFT JOIN tx_results tx ON tx.rowid = e.tx_id
ORDER BY e.height ASC, e.rowid ASC;
"#,
        )
        .bind(first)
        .bind(last)
        .fetch(&self.src)
        .map_ok(|(local_rowid, type_str, height, tx_hash, attrs)| {
            tracing::debug!(?local_rowid, type_str, ?height, ?tx_hash);
            let tx_hash: Option<[u8; 32]> = tx_hash.map(|s| {
                hex::decode(s)
                    .expect("invalid tx_hash")
                    .try_into()
                    .expect("expected 32 bytes")
            });
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
                        // we never hit this because of how we constructed the query
                        _ => None,
                    })
                    .map(Into::into)
                    .collect(),
            };
            (height, event, tx_hash, local_rowid)
        })
        .map_err(|e| anyhow::Error::from(e).context("error reading from database"))
    }

    pub async fn event_batch(&self, first: Height, last: Height) -> anyhow::Result<EventBatch> {
        let mut out = (u64::from(first)..=u64::from(last))
            .map(|height| BlockEvents::new(height))
            .collect::<Vec<_>>();
        let mut tx_stream = self.transactions_between(first, last).boxed();
        while let Some((height, tx_hash, tx_data)) = tx_stream.try_next().await? {
            out[(height.0 - first.0) as usize].push_tx(tx_hash, tx_data);
        }
        let mut events_stream = self.events_between(first, last).boxed();
        while let Some((height, event, tx_hash, local_rowid)) = events_stream.try_next().await? {
            out[(height.0 - first.0) as usize].push_event(event, tx_hash, local_rowid);
        }
        Ok(EventBatch::new(out))
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
