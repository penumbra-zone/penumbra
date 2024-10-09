use cometindex::{async_trait, AppView, ContextualizedEvent, PgTransaction};
use penumbra_dex::event::EventCandlestickData;
use penumbra_proto::DomainType;
use prost::Name as _;
use sqlx::PgPool;

#[derive(Debug)]
pub struct Component {}

impl Component {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl AppView for Component {
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

    fn is_relevant(&self, type_str: &str) -> bool {
        [<EventCandlestickData as DomainType>::Proto::NAME]
            .into_iter()
            .any(|x| type_str == x)
    }

    async fn index_event(
        &self,
        _dbtx: &mut PgTransaction,
        _event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }
}
