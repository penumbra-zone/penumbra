use anyhow::anyhow;
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};
use penumbra_proto::{core::component::sct::v1 as pb, event::ProtoEvent};
use sqlx::{types::chrono::DateTime, PgPool};

#[derive(Debug)]
pub struct Block {}

#[async_trait]
impl AppView for Block {
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

    fn is_relevant(&self, type_str: &str) -> bool {
        type_str == "penumbra.core.component.sct.v1.EventBlockRoot"
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> Result<(), anyhow::Error> {
        let pe = pb::EventBlockRoot::from_event(event.as_ref())?;
        let timestamp = pe
            .timestamp
            .ok_or(anyhow!("block at height {} has no timestamp", pe.height))?;

        sqlx::query(
            "
            INSERT INTO block_details (height, timestamp, root)
            VALUES ($1, $2, $3)
            ",
        )
        .bind(i64::try_from(pe.height)?)
        .bind(
            DateTime::from_timestamp(timestamp.seconds, u32::try_from(timestamp.nanos)?)
                .ok_or(anyhow!("failed to convert timestamp"))?,
        )
        .bind(pe.root.unwrap().inner)
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }
}
