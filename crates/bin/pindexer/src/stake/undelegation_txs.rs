use anyhow::{anyhow, Result};
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgPool, PgTransaction};
use penumbra_num::Amount;
use penumbra_proto::{core::component::stake::v1 as pb, event::ProtoEvent};
use penumbra_stake::IdentityKey;

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
                ik TEXT NOT NULL,
                amount BIGINT NOT NULL,
                height BIGINT NOT NULL,
                tx_hash BYTEA NOT NULL
            );",
        )
        .execute(dbtx.as_mut())
        .await?;

        // Create index on ik
        sqlx::query("CREATE INDEX idx_stake_undelegation_txs_ik ON stake_undelegation_txs(ik);")
            .execute(dbtx.as_mut())
            .await?;

        // Create descending index on height
        sqlx::query(
            "CREATE INDEX idx_stake_undelegation_txs_height ON stake_undelegation_txs(height DESC);",
        )
        .execute(dbtx.as_mut())
        .await?;

        // Create composite index on ik and height (descending)
        sqlx::query("CREATE INDEX idx_stake_undelegation_txs_ik_height ON stake_undelegation_txs(ik, height DESC);")
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

        let ik: IdentityKey = pe
            .identity_key
            .ok_or_else(|| anyhow::anyhow!("missing ik in event"))?
            .try_into()?;

        let amount = Amount::try_from(
            pe.amount
                .ok_or_else(|| anyhow::anyhow!("missing amount in event"))?,
        )?;

        sqlx::query(
            "INSERT INTO stake_undelegation_txs (ik, amount, height, tx_hash) VALUES ($1, $2, $3, $4)"
        )
        .bind(ik.to_string())
        .bind(amount.value() as i64)
        .bind(event.block_height as i64)
        .bind(event.tx_hash.ok_or_else(|| anyhow!("missing tx hash in event"))?)
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }
}
