use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};
use penumbra_proto::{core::component::dex::v1 as pb, event::ProtoEvent};

#[derive(Debug)]
pub struct Lp {}

#[async_trait]
impl AppView for Lp {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            "
CREATE TABLE IF NOT EXISTS lp_updates (
    id SERIAL PRIMARY KEY,
    height INT8 NOT NULL,
    type integer NOT NULL,
    position_id BYTEA NOT NULL,
    trading_pair BYTEA,

);
",
        )
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        [
            "penumbra.core.component.dex.v1.EventPositionWithdraw",
            "penumbra.core.component.dex.v1.EventPositionOpen",
            "penumbra.core.component.dex.v1.EventPositionClose",
        ]
        .contains(&type_str)
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
    ) -> Result<(), anyhow::Error> {
        match event.event.kind.as_str() {
            "penumbra.core.component.dex.v1.EventPositionOpen" => {
                let pe = pb::EventPositionOpen::from_event(event.as_ref())?;

                sqlx::query(
                    "
            INSERT INTO lp_updates (height, type, position_id)
            VALUES ($1, $2, $3)
            ",
                )
                .bind(event.block_height as i64)
                .bind(0)
                .bind(pe.position_id.expect("Position id not found").inner)
                .execute(dbtx.as_mut())
                .await?;
            }
            "penumbra.core.component.dex.v1.EventPositionClose" => {
                let pe = pb::EventPositionClose::from_event(event.as_ref())?;

                sqlx::query(
                    "
            INSERT INTO lp_updates (height, type, position_id)
            VALUES ($1, $2, $3)
            ",
                )
                .bind(event.block_height as i64)
                .bind(1)
                .bind(pe.position_id.expect("Position id not found").inner)
                .execute(dbtx.as_mut())
                .await?;
            }
            "penumbra.core.component.dex.v1.EventPositionWithdraw" => {
                let pe = pb::EventPositionWithdraw::from_event(event.as_ref())?;

                sqlx::query(
                    "
            INSERT INTO lp_updates (height, type, position_id)
            VALUES ($1, $2, $3)
            ",
                )
                .bind(event.block_height as i64)
                .bind(2)
                .bind(pe.position_id.expect("Position id not found").inner)
                .execute(dbtx.as_mut())
                .await?;
            }
            _ => {}
        }

        Ok(())
    }
}
