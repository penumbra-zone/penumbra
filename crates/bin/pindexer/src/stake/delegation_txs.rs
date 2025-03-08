use anyhow::{anyhow, Result};
use cometindex::{
    async_trait,
    index::{EventBatch, EventBatchContext},
    sqlx, AppView, PgTransaction,
};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{core::component::stake::v1 as pb, event::ProtoEvent};
use penumbra_sdk_stake::IdentityKey;

#[derive(Debug)]
pub struct DelegationTxs {}

#[async_trait]
impl AppView for DelegationTxs {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> Result<()> {
        // Create the table
        sqlx::query(
            "CREATE TABLE stake_delegation_txs (
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
        sqlx::query("CREATE INDEX idx_stake_delegation_txs_ik ON stake_delegation_txs(ik);")
            .execute(dbtx.as_mut())
            .await?;

        // Create descending index on height
        sqlx::query(
            "CREATE INDEX idx_stake_delegation_txs_height ON stake_delegation_txs(height DESC);",
        )
        .execute(dbtx.as_mut())
        .await?;

        // Create composite index on ik and height (descending)
        sqlx::query("CREATE INDEX idx_stake_delegation_txs_validator_ik_height ON stake_delegation_txs(ik, height DESC);")
            .execute(dbtx.as_mut())
            .await?;

        Ok(())
    }

    fn name(&self) -> String {
        "stake/delegation_txs".to_string()
    }

    async fn index_batch(
        &self,
        dbtx: &mut PgTransaction,
        batch: EventBatch,
        _ctx: EventBatchContext,
    ) -> Result<()> {
        for event in batch.events() {
            let pe = match pb::EventDelegate::from_event(event.as_ref()) {
                Ok(pe) => pe,
                Err(_) => continue,
            };

            let ik: IdentityKey = pe
                .identity_key
                .ok_or_else(|| anyhow::anyhow!("missing ik in event"))?
                .try_into()?;

            let amount = Amount::try_from(
                pe.amount
                    .ok_or_else(|| anyhow::anyhow!("missing amount in event"))?,
            )?;

            sqlx::query(
            "INSERT INTO stake_delegation_txs (ik, amount, height, tx_hash) VALUES ($1, $2, $3, $4)"
        )
        .bind(ik.to_string())
        .bind(amount.value() as i64)
        .bind(event.block_height as i64)
        .bind(event.tx_hash().ok_or_else(|| anyhow!("missing tx hash in event"))?)
        .execute(dbtx.as_mut())
        .await?;
        }

        Ok(())
    }
}
