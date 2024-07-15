use anyhow::{anyhow, Result};
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction, PgPool};
use penumbra_num::Amount;
use penumbra_proto::{core::component::stake::v1 as pb, event::ProtoEvent};

#[derive(Debug)]
pub struct UndelegationTxs {}

#[async_trait]
impl AppView for UndelegationTxs {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> Result<()> {
        // Create the table
        sqlx::query(
            "CREATE TABLE stake_undelegation_txs (
                id SERIAL PRIMARY KEY,
                validator_ik BYTEA NOT NULL,
                amount BIGINT NOT NULL,
                height BIGINT NOT NULL,
                tx_hash BYTEA NOT NULL
            );",
        )
        .execute(dbtx.as_mut())
        .await?;

        // Create index on validator_ik
        sqlx::query("CREATE INDEX idx_stake_undelegation_txs_validator_ik ON stake_undelegation_txs(validator_ik);")
            .execute(dbtx.as_mut())
            .await?;

        // Create descending index on height
        sqlx::query(
            "CREATE INDEX idx_stake_undelegation_txs_height ON stake_undelegation_txs(height DESC);",
        )
        .execute(dbtx.as_mut())
        .await?;

        // Create composite index on validator_ik and height (descending)
        sqlx::query("CREATE INDEX idx_stake_undelegation_txs_validator_ik_height ON stake_undelegation_txs(validator_ik, height DESC);")
            .execute(dbtx.as_mut())
            .await?;

        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        type_str == "penumbra.core.component.stake.v1.EventUndelegate"
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> Result<()> {
        let pe = pb::EventUndelegate::from_event(event.as_ref())?;

        let ik_bytes = pe
            .identity_key
            .ok_or_else(|| anyhow::anyhow!("missing ik in event"))?
            .ik;

        let amount = Amount::try_from(
            pe.amount
                .ok_or_else(|| anyhow::anyhow!("missing amount in event"))?,
        )?;

        sqlx::query(
            "INSERT INTO stake_undelegation_txs (validator_ik, amount, height, tx_hash) VALUES ($1, $2, $3, $4)"
        )
        .bind(&ik_bytes)
        .bind(amount.value() as i64)
        .bind(event.block_height as i64)
        .bind(event.tx_hash.ok_or_else(|| anyhow!("missing tx hash in event"))?)
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }
}
