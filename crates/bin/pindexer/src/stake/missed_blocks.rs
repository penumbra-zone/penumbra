use anyhow::Result;
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgPool, PgTransaction};

use penumbra_proto::{core::component::stake::v1 as pb, event::ProtoEvent};

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
                ik BYTEA NOT NULL
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

    fn is_relevant(&self, type_str: &str) -> bool {
        type_str == "penumbra.core.component.stake.v1.EventValidatorMissedBlock"
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> Result<(), anyhow::Error> {
        let pe = pb::EventValidatorMissedBlock::from_event(event.as_ref())?;
        let ik_bytes = pe
            .identity_key
            .ok_or_else(|| anyhow::anyhow!("missing ik in event"))?
            .ik;

        let height = event.block_height;

        sqlx::query("INSERT INTO stake_missed_blocks (height, ik) VALUES ($1, $2)")
            .bind(height as i64)
            .bind(ik_bytes)
            .execute(dbtx.as_mut())
            .await?;

        Ok(())
    }
}
