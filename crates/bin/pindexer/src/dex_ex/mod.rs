use std::fmt::Display;

use anyhow::{anyhow, Context};
use chrono::{Datelike, Days, TimeZone, Timelike as _, Utc};
use cometindex::{async_trait, AppView, ContextualizedEvent, PgTransaction};
use penumbra_asset::asset;
use penumbra_dex::{event::EventCandlestickData, CandlestickData};
use penumbra_proto::{event::EventDomainType, DomainType};
use penumbra_sct::event::EventBlockRoot;
use prost::Name as _;
use sqlx::PgPool;

type DateTime = sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>;

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

#[derive(Clone, Copy, Debug)]
enum Window {
    W1m,
    W15m,
    W1h,
    W4h,
    W1d,
    W1w,
    W1mo,
}

impl Window {
    fn all() -> impl Iterator<Item = Self> {
        [
            Window::W1m,
            Window::W15m,
            Window::W1h,
            Window::W4h,
            Window::W1d,
            Window::W1w,
            Window::W1mo,
        ]
        .into_iter()
    }

    /// Get the anchor for a given time.
    ///
    /// This is the latest time that "snaps" to a given anchor, dependent on the window.
    ///
    /// For example, the 1 minute window has an anchor every minute, the day window
    /// every day, etc.
    fn anchor(&self, time: DateTime) -> DateTime {
        let (y, mo, d, h, m) = (
            time.year(),
            time.month(),
            time.day(),
            time.hour(),
            time.minute(),
        );
        let out = match self {
            Window::W1m => Utc.with_ymd_and_hms(y, mo, d, h, m, 0).single(),
            Window::W15m => Utc.with_ymd_and_hms(y, mo, d, h, m - (m % 15), 0).single(),
            Window::W1h => Utc.with_ymd_and_hms(y, mo, d, h, 0, 0).single(),
            Window::W4h => Utc.with_ymd_and_hms(y, mo, d, h - (h % 4), 0, 0).single(),
            Window::W1d => Utc.with_ymd_and_hms(y, mo, d, 0, 0, 0).single(),
            Window::W1w => Utc
                .with_ymd_and_hms(y, mo, d, 0, 0, 0)
                .single()
                .and_then(|x| {
                    x.checked_sub_days(Days::new(time.weekday().num_days_from_monday().into()))
                }),
            Window::W1mo => Utc.with_ymd_and_hms(y, mo, 1, 0, 0, 0).single(),
        };
        out.unwrap()
    }
}

impl Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Window::*;
        let str = match self {
            W1m => "1m",
            W15m => "15m",
            W1h => "1h",
            W4h => "4h",
            W1d => "1d",
            W1w => "1w",
            W1mo => "1mo",
        };
        write!(f, "{}", str)
    }
}

mod price_chart {
    use super::*;

    /// A context when processing a price chart.
    #[derive(Debug)]
    pub struct Context<'tx, 'db> {
        dbtx: &'tx mut PgTransaction<'db>,
        asset_start: asset::Id,
        asset_end: asset::Id,
        window: Window,
    }

    impl<'tx, 'db> Context<'tx, 'db> {
        pub fn new(
            dbtx: &'tx mut PgTransaction<'db>,
            asset_start: asset::Id,
            asset_end: asset::Id,
            window: Window,
        ) -> Self {
            Self {
                dbtx,
                asset_start,
                asset_end,
                window,
            }
        }

        /// Get the candle we should update, based on the current timestamp.
        async fn relevant_candle(
            &mut self,
            anchor: DateTime,
        ) -> anyhow::Result<Option<(i32, Candle)>> {
            let stuff: Option<(i32, f64, f64, f64, f64, f64, f64)> = sqlx::query_as(
                r#"
            SELECT
                dex_ex_candlesticks.id,
                open,
                close,
                high,
                low,
                direct_volume,
                swap_volume
            FROM dex_ex_price_charts 
            JOIN dex_ex_candlesticks ON dex_ex_candlesticks.id = candlestick_id 
            WHERE asset_start = $1
            AND asset_end = $2
            AND the_window = $3
            AND start_time >= $4
            "#,
            )
            .bind(self.asset_start.to_bytes().as_slice())
            .bind(self.asset_end.to_bytes().as_slice())
            .bind(self.window.to_string())
            .bind(anchor)
            .fetch_optional(self.dbtx.as_mut())
            .await?;
            Ok(
                stuff.map(|(id, open, close, high, low, direct_volume, swap_volume)| {
                    (
                        id,
                        Candle {
                            open,
                            close,
                            high,
                            low,
                            direct_volume,
                            swap_volume,
                        },
                    )
                }),
            )
        }

        async fn create_candle(&mut self, anchor: DateTime, candle: Candle) -> anyhow::Result<()> {
            let id: i32 = sqlx::query_scalar(
                r#"
                INSERT INTO dex_ex_candlesticks VALUES (DEFAULT, $1, $2, $3, $4, $5, $6) RETURNING id
            "#,
            )
            .bind(candle.open)
            .bind(candle.close)
            .bind(candle.high)
            .bind(candle.low)
            .bind(candle.direct_volume)
            .bind(candle.swap_volume)
            .fetch_one(self.dbtx.as_mut())
            .await?;
            sqlx::query(
                r#"
                INSERT INTO dex_ex_price_charts VALUES (DEFAULT, $1, $2, $3, $4, $5)
            "#,
            )
            .bind(self.asset_start.to_bytes().as_slice())
            .bind(self.asset_end.to_bytes().as_slice())
            .bind(self.window.to_string())
            .bind(anchor)
            .bind(id)
            .execute(self.dbtx.as_mut())
            .await?;
            Ok(())
        }

        async fn update_candle(&mut self, id: i32, candle: Candle) -> anyhow::Result<()> {
            sqlx::query(
                r#"
                UPDATE dex_ex_candlesticks 
                SET (open, close, high, low, direct_volume, swap_volume) = 
                    ($1, $2, $3, $4, $5, $6)
                WHERE id = $7
            "#,
            )
            .bind(candle.open)
            .bind(candle.close)
            .bind(candle.high)
            .bind(candle.low)
            .bind(candle.direct_volume)
            .bind(candle.swap_volume)
            .bind(id)
            .execute(self.dbtx.as_mut())
            .await?;
            Ok(())
        }

        pub async fn update(&mut self, time: DateTime, candle: Candle) -> anyhow::Result<()> {
            let anchor = self.window.anchor(time);
            match self.relevant_candle(anchor).await? {
                None => self.create_candle(anchor, candle).await?,
                Some((id, old_candle)) => self.update_candle(id, old_candle.merge(&candle)).await?,
            };
            Ok(())
        }
    }
}

use price_chart::Context as PriceChartContext;

mod summary {
    use super::*;

    #[derive(Debug)]
    pub struct Context<'tx, 'db> {
        dbtx: &'tx mut PgTransaction<'db>,
        asset_start: asset::Id,
        asset_end: asset::Id,
    }

    impl<'tx, 'db> Context<'tx, 'db> {
        pub fn new(
            dbtx: &'tx mut PgTransaction<'db>,
            asset_start: asset::Id,
            asset_end: asset::Id,
        ) -> Self {
            Self {
                dbtx,
                asset_start,
                asset_end,
            }
        }

        pub async fn add_candle(&mut self, time: DateTime, candle: Candle) -> anyhow::Result<()> {
            let asset_start = self.asset_start.to_bytes();
            let asset_end = self.asset_end.to_bytes();
            sqlx::query(
                r#"
                INSERT INTO _dex_ex_summary_backing VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            )
            .bind(asset_start.as_slice())
            .bind(asset_end.as_slice())
            .bind(time)
            .bind(candle.close)
            .bind(candle.direct_volume)
            .bind(candle.swap_volume)
            .execute(self.dbtx.as_mut())
            .await?;
            Ok(())
        }
    }

    pub async fn update_all(dbtx: &mut PgTransaction<'_>, time: DateTime) -> anyhow::Result<()> {
        let time_24h_ago = time
            .checked_sub_days(Days::new(1))
            .ok_or(anyhow!("should be able to get time 24h ago from {}", time))?;
        sqlx::query(
            r#"
            DELETE FROM _dex_ex_summary_backing WHERE time < $1
        "#,
        )
        .bind(time_24h_ago)
        .execute(dbtx.as_mut())
        .await?;
        // Update all of the summaries with relevant backing data.
        //
        // We choose this one as being responsible for creating the first summary.
        sqlx::query(
            r#"
            INSERT INTO dex_ex_summary
            SELECT DISTINCT ON (asset_start, asset_end) 
              asset_start, 
              asset_end, 
              FIRST_VALUE(price) OVER w AS price_24h_ago, 
              price AS current_price, 
              MAX(price) OVER w AS high_24h, 
              MIN(price) OVER w AS low_24h, 
              SUM(direct_volume) OVER w AS direct_volume_24h, 
              SUM(swap_volume) OVER w AS swap_volume_24h 
            FROM _dex_ex_summary_backing 
            WINDOW w AS (
              PARTITION BY 
                asset_start, asset_end 
              ORDER BY asset_start, asset_end, time DESC
            ) ORDER by asset_start, asset_end, time ASC
            ON CONFLICT (asset_start, asset_end) DO UPDATE SET
                price_24h_ago = EXCLUDED.price_24h_ago,
                current_price = EXCLUDED.current_price, 
                high_24h = EXCLUDED.high_24h, 
                low_24h = EXCLUDED.low_24h, 
                direct_volume_24h = EXCLUDED.direct_volume_24h, 
                swap_volume_24h = EXCLUDED.swap_volume_24h
        "#,
        )
        .execute(dbtx.as_mut())
        .await?;
        // When we don't have backing data, we should nonetheless update to reflect this
        sqlx::query(
            r#"
            UPDATE dex_ex_summary
            SET
                price_24h_ago = current_price,
                high_24h = current_price,
                low_24h = current_price,
                direct_volume_24h = 0,
                swap_volume_24h = 0
            WHERE NOT EXISTS (
                SELECT 1
                FROM _dex_ex_summary_backing
                WHERE
                    _dex_ex_summary_backing.asset_start = dex_ex_summary.asset_start
                AND
                    _dex_ex_summary_backing.asset_end = dex_ex_summary.asset_end
            )
        "#,
        )
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }
}

use summary::Context as SummaryContext;

async fn queue_event_candlestick_data(
    dbtx: &mut PgTransaction<'_>,
    height: u64,
    event: EventCandlestickData,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO _dex_ex_queue VALUES (DEFAULT, $1, $2)")
        .bind(i64::try_from(height)?)
        .bind(event.encode_to_vec().as_slice())
        .execute(dbtx.as_mut())
        .await?;
    Ok(())
}

async fn unqueue_event_candlestick_data(
    dbtx: &mut PgTransaction<'_>,
    height: u64,
) -> anyhow::Result<Vec<EventCandlestickData>> {
    let values: Vec<Vec<u8>> =
        sqlx::query_scalar("DELETE FROM _dex_ex_queue WHERE height = $1 RETURNING data")
            .bind(i64::try_from(height)?)
            .fetch_all(dbtx.as_mut())
            .await?;
    values
        .into_iter()
        .map(|x| EventCandlestickData::decode(x.as_slice()))
        .collect()
}

async fn on_event_candlestick_data(
    dbtx: &mut PgTransaction<'_>,
    event_time: DateTime,
    event: EventCandlestickData,
) -> anyhow::Result<()> {
    let asset_start = event.pair.start;
    let asset_end = event.pair.end;
    let candle = event.stick.into();
    for window in Window::all() {
        let mut ctx = PriceChartContext::new(dbtx, asset_start, asset_end, window);
        ctx.update(event_time, candle).await?;
    }
    let mut ctx = SummaryContext::new(dbtx, asset_start, asset_end);
    ctx.add_candle(event_time, candle).await?;
    Ok(())
}

async fn fetch_height_time(
    dbtx: &mut PgTransaction<'_>,
    height: u64,
) -> anyhow::Result<Option<DateTime>> {
    const CTX: &'static str = r#"
The `dex_ex` component relies on the `block` component to be running, to provide the `block_details` with timestamps. 
Make sure that is running as well.
"#;
    sqlx::query_scalar("SELECT timestamp FROM block_details WHERE height = $1")
        .bind(i64::try_from(height)?)
        .fetch_optional(dbtx.as_mut())
        .await
        .context(CTX)
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
        [
            <EventCandlestickData as DomainType>::Proto::full_name(),
            <EventBlockRoot as DomainType>::Proto::full_name(),
        ]
        .into_iter()
        .any(|x| type_str == x)
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> Result<(), anyhow::Error> {
        if let Ok(e) = EventCandlestickData::try_from_event(&event.event) {
            let height = event.block_height;
            match fetch_height_time(dbtx, height).await? {
                None => {
                    queue_event_candlestick_data(dbtx, height, e).await?;
                }
                Some(time) => {
                    on_event_candlestick_data(dbtx, time, e).await?;
                }
            }
        } else if let Ok(e) = EventBlockRoot::try_from_event(&event.event) {
            let height = e.height;
            let time = DateTime::from_timestamp(e.timestamp_seconds, 0).ok_or(anyhow!(
                "creating timestamp should succeed; timestamp: {}",
                e.timestamp_seconds
            ))?;
            for event in unqueue_event_candlestick_data(dbtx, height).await? {
                on_event_candlestick_data(dbtx, time, event).await?;
            }
            summary::update_all(dbtx, time).await?;
        }
        tracing::debug!(?event, "unrecognized event");
        Ok(())
    }
}
