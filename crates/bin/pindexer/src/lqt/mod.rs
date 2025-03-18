use anyhow::anyhow;
use cometindex::{
    async_trait,
    index::{EventBatch, EventBatchContext},
    sqlx, AppView, ContextualizedEvent, PgTransaction,
};
use penumbra_sdk_funding::event::{EventLqtDelegatorReward, EventLqtPositionReward, EventLqtVote};
use penumbra_sdk_proto::event::EventDomainType;

#[derive(Debug)]
pub struct Lqt {}

impl Lqt {
    async fn index_event(
        &self,
        dbtx: &mut PgTransaction<'_>,
        event: &ContextualizedEvent,
    ) -> anyhow::Result<()> {
        if let Ok(e) = EventLqtVote::try_from_event(&event.event) {
            sqlx::query("INSERT INTO lqt_votes VALUES (DEFAULT, $1, $2, $3, $4, $5)")
                .bind(i64::try_from(e.epoch_index)?)
                .bind(&e.incentivized_asset_id.to_bytes())
                .bind(i64::try_from(e.voting_power.value())?)
                .bind(&e.tx_id.0)
                .bind(&e.rewards_recipient.to_vec())
                .execute(dbtx.as_mut())
                .await?;
        } else if let Ok(e) = EventLqtDelegatorReward::try_from_event(&event.event) {
            sqlx::query("INSERT INTO lqt_delegator_rewards VALUES (DEFAULT, $1, $2, $3, $4)")
                .bind(i64::try_from(e.epoch_index)?)
                .bind(i64::try_from(e.reward_amount.value())?)
                .bind(&e.rewards_recipient.to_vec())
                .bind(&e.incentivized_asset_id.to_bytes())
                .execute(dbtx.as_mut())
                .await?;
        } else if let Ok(e) = EventLqtPositionReward::try_from_event(&event.event) {
            sqlx::query("INSERT INTO lqt_delegator_rewards VALUES (DEFAULT, $1, $2, $3, $4, $5::NUMERIC, $6::NUMERIC)")
                .bind(i64::try_from(e.epoch_index)?)
                .bind(i64::try_from(e.reward_amount.value())?)
                .bind(&e.position_id.0)
                .bind(&e.incentivized_asset_id.to_bytes())
                .bind(e.tournament_volume.to_string())
                .bind(e.position_volume.to_string())
                .execute(dbtx.as_mut())
                .await?;
        }
    }
}

#[async_trait]
impl AppView for Lqt {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        for statement in include_str!("schema.sql").split(";") {
            sqlx::query(statement).execute(dbtx.as_mut()).await?;
        }
        Ok(())
    }

    fn name(&self) -> String {
        "lqt".to_string()
    }

    async fn index_batch(
        &self,
        dbtx: &mut PgTransaction,
        batch: EventBatch,
        _ctx: EventBatchContext,
    ) -> Result<(), anyhow::Error> {
        for event in batch.events() {
            self.index_event(dbtx, event).await?;
        }
        Ok(())
    }
}
