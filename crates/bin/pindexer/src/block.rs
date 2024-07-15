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
    id SERIAL PRIMARY KEY,
    root BYTEA NOT NULL,
    height INT8 NOT NULL,
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
        let timestamp = pe.timestamp.expect("Block has no timestamp");

        sqlx::query(
            "
            INSERT INTO block_details (height, timestamp, root)
            VALUES ($1, $2, $3)
            ",
        )
        .bind(pe.height as i64)
        .bind(DateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32).unwrap())
        .bind(pe.root.unwrap().inner)
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }
}
