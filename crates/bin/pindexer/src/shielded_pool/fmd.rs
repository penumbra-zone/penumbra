use cometindex::{async_trait, sqlx, ContextualizedEvent, Index, PgTransaction};
use penumbra_proto::{core::component::shielded_pool::v1 as pb, event::ProtoEvent};

#[derive(Debug)]
pub struct ClueSet {}

#[async_trait]
impl Index for ClueSet {
    async fn create_tables(&self, dbtx: &mut PgTransaction) -> Result<(), anyhow::Error> {
        sqlx::query(
            // table name is module path + struct name
            "
CREATE TABLE IF NOT EXISTS shielded_pool_fmd_clue_set (
    id SERIAL PRIMARY KEY,
    clue_bytes BYTEA NOT NULL,
    tx_hash BYTEA NOT NULL
);
",
        )
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        type_str == "penumbra.core.component.shielded_pool.v1.EventBroadcastClue"
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
    ) -> Result<(), anyhow::Error> {
        let pe = pb::EventBroadcastClue::from_event(event.as_ref())?;

        let clue_bytes = pe
            .clue
            .ok_or_else(|| anyhow::anyhow!("clue event missing clue"))?
            .inner;

        let tx_hash = event.tx_hash.as_ref().expect("tx_hash not found").to_vec();

        sqlx::query(
            "
            INSERT INTO shielded_pool_fmd_clue_set (clue_bytes, tx_hash)
            VALUES ($1, $2)
            ",
        )
        .bind(&clue_bytes)
        .bind(&tx_hash)
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }
}
