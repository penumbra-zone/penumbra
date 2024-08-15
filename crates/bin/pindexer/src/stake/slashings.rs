use anyhow::{anyhow, Result};
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgPool, PgTransaction};

use penumbra_proto::{core::component::stake::v1 as pb, event::ProtoEvent};
use penumbra_stake::IdentityKey;

#[derive(Debug)]
pub struct Slashings {}

#[async_trait]
impl AppView for Slashings {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            "CREATE TABLE stake_slashings (
                id SERIAL PRIMARY KEY,
                height BIGINT NOT NULL,
                ik TEXT NOT NULL,
                epoch_index BIGINT NOT NULL,
                penalty TEXT NOT NULL
            );",
        )
        .execute(dbtx.as_mut())
        .await?;

        sqlx::query("CREATE INDEX idx_stake_slashings_height ON stake_slashings(height);")
            .execute(dbtx.as_mut())
            .await?;

        sqlx::query("CREATE INDEX idx_stake_slashings_ik ON stake_slashings(ik);")
            .execute(dbtx.as_mut())
            .await?;

        sqlx::query("CREATE INDEX idx_stake_slashings_height_ik ON stake_slashings(height, ik);")
            .execute(dbtx.as_mut())
            .await?;

        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        match type_str {
            "penumbra.core.component.stake.v1.EventSlashingPenaltyApplied" => true,
            _ => false,
        }
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> Result<(), anyhow::Error> {
        let pe = pb::EventSlashingPenaltyApplied::from_event(event.as_ref())?;
        let ik = IdentityKey::try_from(
            pe.identity_key
                .ok_or_else(|| anyhow!("missing ik in event"))?,
        )?;

        let height = event.block_height;
        let epoch_index = pe.epoch_index;

        let penalty_json = serde_json::to_string(
            &pe.new_penalty
                .ok_or_else(|| anyhow!("missing new_penalty"))?,
        )?;

        sqlx::query(
            "INSERT INTO stake_slashings (height, ik, epoch_index, penalty) 
             VALUES ($1, $2, $3, $4)",
        )
        .bind(height as i64)
        .bind(ik.to_string())
        .bind(epoch_index as i64)
        .bind(penalty_json)
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }
}
