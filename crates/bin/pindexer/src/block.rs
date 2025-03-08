use cometindex::{
    async_trait,
    index::{EventBatch, EventBatchContext},
    sqlx, AppView, PgTransaction,
};
use penumbra_sdk_proto::{core::component::sct::v1 as pb, event::ProtoEvent};
use sqlx::types::chrono::DateTime;

#[derive(Debug)]
pub struct Block {}

#[async_trait]
impl AppView for Block {
    fn name(&self) -> String {
        "block".to_string()
    }

    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            // table name is module path + struct name
            "
CREATE TABLE IF NOT EXISTS block_details (
    height BIGINT PRIMARY KEY,
    root BYTEA NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL
);
",
        )
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    async fn index_batch(
        &self,
        dbtx: &mut PgTransaction,
        batch: EventBatch,
        _ctx: EventBatchContext,
    ) -> Result<(), anyhow::Error> {
        for event in batch.events() {
            let pe = match pb::EventBlockRoot::from_event(event.as_ref()) {
                Ok(pe) => pe,
                Err(_) => continue,
            };
            let timestamp = pe.timestamp.unwrap_or_default();

            sqlx::query(
                "
            INSERT INTO block_details (height, timestamp, root)
            VALUES ($1, $2, $3)
            ",
            )
            .bind(i64::try_from(pe.height)?)
            .bind(DateTime::from_timestamp(
                timestamp.seconds,
                u32::try_from(timestamp.nanos)?,
            ))
            .bind(pe.root.unwrap().inner)
            .execute(dbtx.as_mut())
            .await?;
        }
        Ok(())
    }
}
