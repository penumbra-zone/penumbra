use cometindex::{async_trait, AppView, ContextualizedEvent, PgTransaction};
use penumbra_dex::{event::EventCandlestickData, CandlestickData};
use penumbra_proto::DomainType;
use prost::Name as _;
use sqlx::PgPool;

/// Candlestick data, unmoored from the prison of a particular block height.
///
/// In other words, this can represent candlesticks which span arbitrary windows,
/// and not just a single block.
#[derive(Debug, Clone, Copy)]
struct Candle {
    open: f64,
    close: f64,
    low: f64,
    high: f64,
    direct_volume: f64,
    swap_volume: f64,
}

impl Candle {
    fn from_candlestick_data(data: &CandlestickData) -> Self {
        Self {
            open: data.open,
            close: data.close,
            low: data.low,
            high: data.high,
            direct_volume: data.direct_volume,
            swap_volume: data.swap_volume,
        }
    }

    fn merge(&self, that: &Self) -> Self {
        Self {
            open: self.open,
            close: that.close,
            low: self.low.min(that.low),
            high: self.high.max(that.high),
            direct_volume: self.direct_volume + that.direct_volume,
            swap_volume: self.swap_volume + that.swap_volume,
        }
    }
}

impl From<CandlestickData> for Candle {
    fn from(value: CandlestickData) -> Self {
        Self::from(&value)
    }
}

impl From<&CandlestickData> for Candle {
    fn from(value: &CandlestickData) -> Self {
        Self::from_candlestick_data(value)
    }
}

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
