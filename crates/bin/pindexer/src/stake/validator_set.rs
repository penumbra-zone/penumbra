use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};

#[derive(Debug)]
pub struct ValidatorSet {}

#[async_trait]
impl AppView for ValidatorSet {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            // table name is module path + struct name
            "CREATE TABLE IF NOT EXISTS stake_validator_set (
                id SERIAL PRIMARY KEY,
                ik BYTEA NOT NULL,
                name TEXT NOT NULL,
                definition BYTEA NOT NULL,
                voting_power BIGINT NOT NULL,
                queued_delegations BIGINT NOT NULL,
                queued_undelegations BIGINT NOT NULL,
                downtime_slash_count INT NOT NULL,
                validator_state TEXT NOT NULL,
                bonding_state TEXT NOT NULL,
            );",
        )
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        todo!()
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
    ) -> Result<(), anyhow::Error> {
        todo!();
    }
}
