use anyhow::Result;
use cometindex::{
    async_trait,
    index::{EventBatch, EventBatchContext},
    sqlx, AppView, PgTransaction,
};

use penumbra_sdk_proto::{core::component::stake::v1 as pb, event::ProtoEvent};
use penumbra_sdk_stake::IdentityKey;

#[derive(Debug)]
pub struct MissedBlocks {}

#[async_trait]
impl AppView for MissedBlocks {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        // Create the table
        sqlx::query(
            "CREATE TABLE stake_missed_blocks (
                id SERIAL PRIMARY KEY,
                height BIGINT NOT NULL,
                ik TEXT NOT NULL
            );",
        )
        .execute(dbtx.as_mut())
        .await?;

        // Create descending index on height
        sqlx::query(
            "CREATE INDEX idx_stake_missed_blocks_height ON stake_missed_blocks(height DESC);",
        )
        .execute(dbtx.as_mut())
        .await?;

        // Create index on ik
        sqlx::query("CREATE INDEX idx_stake_missed_blocks_ik ON stake_missed_blocks(ik);")
            .execute(dbtx.as_mut())
            .await?;

        // Create composite index on height (descending) and ik
        sqlx::query("CREATE INDEX idx_stake_missed_blocks_height_ik ON stake_missed_blocks(height DESC, ik);")
            .execute(dbtx.as_mut())
            .await?;

        Ok(())
    }

    fn name(&self) -> String {
        "stake/missed_blocks".to_string()
    }

    async fn index_batch(
        &self,
        dbtx: &mut PgTransaction,
        batch: EventBatch,
        _ctx: EventBatchContext,
    ) -> Result<(), anyhow::Error> {
        for event in batch.events() {
            let pe = match pb::EventValidatorMissedBlock::from_event(event.as_ref()) {
                Ok(pe) => pe,
                Err(_) => continue,
            };
            let ik: IdentityKey = pe
                .identity_key
                .ok_or_else(|| anyhow::anyhow!("missing ik in event"))?
                .try_into()?;

            let height = event.block_height;

            sqlx::query("INSERT INTO stake_missed_blocks (height, ik) VALUES ($1, $2)")
                .bind(height as i64)
                .bind(ik.to_string())
                .execute(dbtx.as_mut())
                .await?;
        }

        Ok(())
    }
}
