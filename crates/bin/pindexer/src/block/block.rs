use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};

#[derive(Debug)]
pub struct Block {}

#[async_trait]
impl AppView for Block {
    async fn init_chain(&self, dbtx: &mut PgTransaction, _: &serde_json::Value) -> Result<(), anyhow::Error> {
        sqlx::query(
            // table name is module path + struct name
            "
CREATE TABLE IF NOT EXISTS block_details (
    id SERIAL PRIMARY KEY,
    height BYTEA NOT NULL,
    timestamp BYTEA NOT NULL
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
    ) -> Result<(), anyhow::Error> {
        dbg!(event);

        Ok(())
    }
}
