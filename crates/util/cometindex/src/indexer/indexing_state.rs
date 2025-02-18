use anyhow::anyhow;
use futures::{Stream, StreamExt, TryStreamExt};
use prost::Message as _;
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Transaction};
use std::collections::HashMap;
use tendermint::abci::{self, Event};

use crate::index::{BlockEvents, EventBatch};

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

impl From<Height> for u64 {
    fn from(value: Height) -> Self {
        value.0
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
        value: <Postgres as sqlx::Database>::ValueRef<'r>,
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
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
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
            let height = Height::try_from(row.0)?;
            Ok((height, tx_hash, transaction))
        }

        sqlx::query_as::<_, (i64, String, Vec<u8>)>(
            r#"
SELECT height, tx_hash, tx_result
FROM tx_results
LEFT JOIN LATERAL (
    SELECT height FROM blocks WHERE blocks.rowid = tx_results.block_id LIMIT 1
) ON TRUE
WHERE
    block_id >= (SELECT rowid FROM blocks where height = $1)
AND
    block_id <= (SELECT rowid FROM blocks where height = $2)
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
            //   attach transaction hash?
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
            let height = Height::try_from(height).expect("failed to decode height");
            (height, event, tx_hash, local_rowid)
        })
        .map_err(|e| anyhow::Error::from(e).context("error reading from database"))
    }

    pub async fn event_batch(&self, first: Height, last: Height) -> anyhow::Result<EventBatch> {
        let mut out = (first.0..=last.0)
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
