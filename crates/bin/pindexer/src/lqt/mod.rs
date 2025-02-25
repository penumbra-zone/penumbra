use cometindex::{
    async_trait, index::EventBatch, sqlx, AppView, ContextualizedEvent, PgTransaction,
};
use penumbra_sdk_dex::event::EventLqtPositionVolume;
use penumbra_sdk_funding::event::{EventLqtDelegatorReward, EventLqtPositionReward, EventLqtVote};
use penumbra_sdk_proto::event::EventDomainType;

#[derive(Debug)]
pub struct Lqt {}

impl Lqt {
    async fn index_event(
        &self,
        _dbtx: &mut PgTransaction<'_>,
        event: ContextualizedEvent<'_>,
    ) -> anyhow::Result<()> {
        if let Ok(_e) = EventLqtVote::try_from_event(&event.event) {
            todo!()
        } else if let Ok(_e) = EventLqtDelegatorReward::try_from_event(&event.event) {
            todo!()
        } else if let Ok(_e) = EventLqtPositionReward::try_from_event(&event.event) {
            todo!()
        } else if let Ok(_e) = EventLqtPositionVolume::try_from_event(&event.event) {
            todo!()
        }
        Ok(())
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
    ) -> Result<(), anyhow::Error> {
        for event in batch.events() {
            self.index_event(dbtx, event).await?;
        }
        Ok(())
    }
}
