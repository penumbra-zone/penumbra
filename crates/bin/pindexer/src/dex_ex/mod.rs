use anyhow::anyhow;
use cometindex::{
    async_trait,
    index::{BlockEvents, EventBatch},
    AppView, PgTransaction,
};
use penumbra_sdk_asset::asset;
use penumbra_sdk_dex::lp::position::{Id as PositionId, Position};
use penumbra_sdk_dex::{
    event::{
        EventBatchSwap, EventCandlestickData, EventPositionClose, EventPositionExecution,
        EventPositionOpen, EventPositionWithdraw, EventQueuePositionClose,
    },
    lp::Reserves,
    DirectedTradingPair, SwapExecution, TradingPair,
};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::event::EventDomainType;
use penumbra_sdk_proto::DomainType;
use penumbra_sdk_sct::event::EventBlockRoot;
use sqlx::types::BigDecimal;
use sqlx::Row;
use std::collections::{BTreeMap, HashMap, HashSet};

type DateTime = sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>;

mod candle {
    use super::DateTime;
    use chrono::{Datelike as _, Days, TimeDelta, TimeZone as _, Timelike as _, Utc};
    use penumbra_sdk_dex::CandlestickData;
    use std::fmt::Display;

    fn geo_mean(a: f64, b: f64) -> f64 {
        (a * b).sqrt()
    }

    /// Candlestick data, unmoored from the prison of a particular block height.
    ///
    /// In other words, this can represent candlesticks which span arbitrary windows,
    /// and not just a single block.
    #[derive(Debug, Clone, Copy)]
    pub struct Candle {
        pub open: f64,
        pub close: f64,
        pub low: f64,
        pub high: f64,
        pub direct_volume: f64,
        pub swap_volume: f64,
    }

    impl Candle {
        pub fn from_candlestick_data(data: &CandlestickData) -> Self {
            // The volume is tracked in terms of input asset.
            // We can use the closing price (aka. the common clearing price) to convert
            // the volume to the other direction i.e, the batch swap output.
            Self {
                open: data.open,
                close: data.close,
                low: data.low,
                high: data.high,
                direct_volume: data.direct_volume,
                swap_volume: data.swap_volume,
            }
        }

        pub fn merge(&mut self, that: &Self) {
            self.close = that.close;
            self.low = self.low.min(that.low);
            self.high = self.high.max(that.high);
            self.direct_volume += that.direct_volume;
            self.swap_volume += that.swap_volume;
        }

        /// Mix this candle with a candle going in the opposite direction of the pair.
        pub fn mix(&mut self, op: &Self) {
            // We use the geometric mean, resulting in all the prices in a.mix(b) being
            // the inverse of the prices in b.mix(a), and the volumes being equal.
            self.close /= geo_mean(self.close, op.close);
            self.open /= geo_mean(self.open, op.open);
            self.low = self.low.min(1.0 / op.low);
            self.high = self.high.min(1.0 / op.high);
            // Using the closing price to look backwards at volume.
            self.direct_volume += op.direct_volume / self.close;
            self.swap_volume += op.swap_volume / self.close;
        }

        /// Flip this candle to get the equivalent in the other direction.
        pub fn flip(&self) -> Self {
            Self {
                open: 1.0 / self.open,
                close: 1.0 / self.close,
                low: 1.0 / self.low,
                high: 1.0 / self.high,
                direct_volume: self.direct_volume * self.close,
                swap_volume: self.swap_volume * self.close,
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

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Window {
        W1m,
        W15m,
        W1h,
        W4h,
        W1d,
        W1w,
        W1mo,
    }

    impl Window {
        pub fn all() -> impl Iterator<Item = Self> {
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
        pub fn anchor(&self, time: DateTime) -> DateTime {
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

        pub fn subtract_from(&self, time: DateTime) -> DateTime {
            let delta = match self {
                Window::W1m => TimeDelta::minutes(1),
                Window::W15m => TimeDelta::minutes(15),
                Window::W1h => TimeDelta::hours(1),
                Window::W4h => TimeDelta::hours(4),
                Window::W1d => TimeDelta::days(1),
                Window::W1w => TimeDelta::weeks(1),
                Window::W1mo => TimeDelta::days(30),
            };
            time - delta
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

    #[derive(Debug)]
    pub struct WindowedCandle {
        start: DateTime,
        window: Window,
        candle: Candle,
    }

    impl WindowedCandle {
        pub fn new(now: DateTime, window: Window, candle: Candle) -> Self {
            Self {
                start: window.anchor(now),
                window,
                candle,
            }
        }

        /// Update with a new candlestick, at a given time.
        ///
        /// This may return the old candlestick and its start time, if we should be starting
        /// a new candle, based on the candle for that window having already been closed.
        pub fn with_candle(&mut self, now: DateTime, candle: Candle) -> Option<(DateTime, Candle)> {
            let start = self.window.anchor(now);
            // This candle belongs to the next window!
            if start > self.start {
                let old_start = std::mem::replace(&mut self.start, start);
                let old_candle = std::mem::replace(&mut self.candle, candle);
                Some((old_start, old_candle))
            } else {
                self.candle.merge(&candle);
                None
            }
        }

        pub fn window(&self) -> Window {
            self.window
        }

        pub fn flush(self) -> (DateTime, Candle) {
            (self.start, self.candle)
        }
    }
}
pub use candle::{Candle, Window, WindowedCandle};

mod price_chart {
    use super::*;

    /// A context when processing a price chart.
    #[derive(Debug)]
    pub struct Context {
        asset_start: asset::Id,
        asset_end: asset::Id,
        window: Window,
        state: Option<WindowedCandle>,
    }

    impl Context {
        pub async fn load(
            dbtx: &mut PgTransaction<'_>,
            asset_start: asset::Id,
            asset_end: asset::Id,
            window: Window,
        ) -> anyhow::Result<Self> {
            let row: Option<(f64, f64, f64, f64, f64, f64, DateTime)> = sqlx::query_as(
                "
                SELECT open, close, high, low, direct_volume, swap_volume, start_time
                FROM dex_ex_price_charts
                WHERE asset_start = $1
                AND asset_end = $2
                AND the_window = $3
                ORDER BY start_time DESC
                LIMIT 1
            ",
            )
            .bind(asset_start.to_bytes())
            .bind(asset_end.to_bytes())
            .bind(window.to_string())
            .fetch_optional(dbtx.as_mut())
            .await?;
            let state = row.map(
                |(open, close, low, high, direct_volume, swap_volume, start)| {
                    let candle = Candle {
                        open,
                        close,
                        low,
                        high,
                        direct_volume,
                        swap_volume,
                    };
                    WindowedCandle::new(start, window, candle)
                },
            );
            Ok(Self {
                asset_start,
                asset_end,
                window,
                state,
            })
        }

        async fn write_candle(
            &self,
            dbtx: &mut PgTransaction<'_>,
            start: DateTime,
            candle: Candle,
        ) -> anyhow::Result<()> {
            sqlx::query(
                "
            INSERT INTO dex_ex_price_charts(
                id, asset_start, asset_end, the_window,
                start_time, open, close, high, low, direct_volume, swap_volume
            ) 
            VALUES(
                DEFAULT, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
            )
            ON CONFLICT (asset_start, asset_end, the_window, start_time) DO UPDATE SET
                open = EXCLUDED.open,
                close = EXCLUDED.close,
                high = EXCLUDED.high,
                low = EXCLUDED.low,
                direct_volume = EXCLUDED.direct_volume,
                swap_volume = EXCLUDED.swap_volume
            ",
            )
            .bind(self.asset_start.to_bytes())
            .bind(self.asset_end.to_bytes())
            .bind(self.window.to_string())
            .bind(start)
            .bind(candle.open)
            .bind(candle.close)
            .bind(candle.high)
            .bind(candle.low)
            .bind(candle.direct_volume)
            .bind(candle.swap_volume)
            .execute(dbtx.as_mut())
            .await?;
            Ok(())
        }

        pub async fn update(
            &mut self,
            dbtx: &mut PgTransaction<'_>,
            now: DateTime,
            candle: Candle,
        ) -> anyhow::Result<()> {
            let state = match self.state.as_mut() {
                Some(x) => x,
                None => {
                    self.state = Some(WindowedCandle::new(now, self.window, candle));
                    self.state.as_mut().unwrap()
                }
            };
            if let Some((start, old_candle)) = state.with_candle(now, candle) {
                self.write_candle(dbtx, start, old_candle).await?;
            };
            Ok(())
        }

        pub async fn unload(mut self, dbtx: &mut PgTransaction<'_>) -> anyhow::Result<()> {
            let state = std::mem::replace(&mut self.state, None);
            if let Some(state) = state {
                let (start, candle) = state.flush();
                self.write_candle(dbtx, start, candle).await?;
            }
            Ok(())
        }
    }
}

use price_chart::Context as PriceChartContext;

mod summary {
    use cometindex::PgTransaction;
    use penumbra_sdk_asset::asset;

    use super::{Candle, DateTime, PairMetrics, Window};

    pub struct Context {
        start: asset::Id,
        end: asset::Id,
        price: f64,
        liquidity: f64,
        start_price_indexing_denom: f64,
    }

    impl Context {
        pub async fn load(
            dbtx: &mut PgTransaction<'_>,
            start: asset::Id,
            end: asset::Id,
        ) -> anyhow::Result<Self> {
            let row: Option<(f64, f64, f64)> = sqlx::query_as(
                "
                SELECT price, liquidity, start_price_indexing_denom
                FROM dex_ex_pairs_block_snapshot
                WHERE asset_start = $1
                AND asset_end = $2
                ORDER BY id DESC
                LIMIT 1
            ",
            )
            .bind(start.to_bytes())
            .bind(end.to_bytes())
            .fetch_optional(dbtx.as_mut())
            .await?;
            let (price, liquidity, start_price_indexing_denom) = row.unwrap_or_default();
            Ok(Self {
                start,
                end,
                price,
                liquidity,
                start_price_indexing_denom,
            })
        }

        pub async fn update(
            &mut self,
            dbtx: &mut PgTransaction<'_>,
            now: DateTime,
            candle: Option<Candle>,
            metrics: PairMetrics,
            start_price_indexing_denom: Option<f64>,
        ) -> anyhow::Result<()> {
            if let Some(candle) = candle {
                self.price = candle.close;
            }
            if let Some(price) = start_price_indexing_denom {
                self.start_price_indexing_denom = price;
            }
            self.liquidity += metrics.liquidity_change;

            sqlx::query(
                "
            INSERT INTO dex_ex_pairs_block_snapshot VALUES (
                DEFAULT, $1, $2, $3, $4, $5, $6, $7, $8, $9
            )        
        ",
            )
            .bind(now)
            .bind(self.start.to_bytes())
            .bind(self.end.to_bytes())
            .bind(self.price)
            .bind(self.liquidity)
            .bind(candle.map(|x| x.direct_volume).unwrap_or_default())
            .bind(candle.map(|x| x.swap_volume).unwrap_or_default())
            .bind(self.start_price_indexing_denom)
            .bind(metrics.trades)
            .execute(dbtx.as_mut())
            .await?;
            Ok(())
        }
    }

    pub async fn update_summary(
        dbtx: &mut PgTransaction<'_>,
        now: DateTime,
        start: asset::Id,
        end: asset::Id,
        window: Window,
    ) -> anyhow::Result<()> {
        let then = window.subtract_from(now);
        sqlx::query(
            "
        WITH
        snapshots AS (
            SELECT *
            FROM dex_ex_pairs_block_snapshot
            WHERE asset_start = $1
            AND asset_end = $2
        ),
        previous AS (
            SELECT price AS price_then, liquidity AS liquidity_then
            FROM snapshots
            WHERE time <= $4
            ORDER BY time DESC
            LIMIT 1
        ),
        previous_or_default AS (
            SELECT
                COALESCE((SELECT price_then FROM previous), 0.0) AS price_then,
                COALESCE((SELECT liquidity_then FROM previous), 0.0) AS liquidity_then
        ),
        now AS (
            SELECT price, liquidity            
            FROM snapshots
            WHERE time >= $3
            ORDER BY time ASC
            LIMIT 1
        ),
        sums AS (
            SELECT
                COALESCE(SUM(direct_volume), 0.0) AS direct_volume_over_window,
                COALESCE(SUM(swap_volume), 0.0) AS swap_volume_over_window,
                COALESCE(SUM(COALESCE(start_price_indexing_denom, 0.0) * direct_volume), 0.0) as direct_volume_indexing_denom_over_window,
                COALESCE(SUM(COALESCE(start_price_indexing_denom, 0.0) * swap_volume), 0.0) as swap_volume_indexing_denom_over_window,
                COALESCE(SUM(trades), 0.0) AS trades_over_window,
                COALESCE(MIN(price), 0.0) AS low,
                COALESCE(MAX(price), 0.0) AS high
            FROM snapshots
            WHERE time <= $3
            AND time >= $4
        )
        INSERT INTO dex_ex_pairs_summary
        SELECT 
            $1, $2, $5,
            price, price_then,
            low, high,
            liquidity, liquidity_then,
            direct_volume_over_window,
            swap_volume_over_window,
            direct_volume_indexing_denom_over_window,
            swap_volume_indexing_denom_over_window,
            trades_over_window
        FROM previous_or_default JOIN now ON TRUE JOIN sums ON TRUE
        ON CONFLICT (asset_start, asset_end, the_window)
        DO UPDATE SET
            price = EXCLUDED.price,
            price_then = EXCLUDED.price_then,
            liquidity = EXCLUDED.liquidity,
            liquidity_then = EXCLUDED.liquidity_then,
            direct_volume_over_window = EXCLUDED.direct_volume_over_window,
            swap_volume_over_window = EXCLUDED.swap_volume_over_window,
            direct_volume_indexing_denom_over_window = EXCLUDED.direct_volume_indexing_denom_over_window,
            swap_volume_indexing_denom_over_window = EXCLUDED.swap_volume_indexing_denom_over_window,
            trades_over_window = EXCLUDED.trades_over_window
        ",
        )
        .bind(start.to_bytes())
        .bind(end.to_bytes())
        .bind(now)
        .bind(then)
        .bind(window.to_string())
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    pub async fn update_aggregate_summary(
        dbtx: &mut PgTransaction<'_>,
        window: Window,
        denom: asset::Id,
        min_liquidity: f64,
    ) -> anyhow::Result<()> {
        // TODO: do something here
        sqlx::query(
            "
        WITH       
            eligible_denoms AS (
                SELECT asset_start as asset, price
                FROM dex_ex_pairs_summary
                WHERE asset_end = $1
                UNION VALUES ($1, 1.0)
            ),
            converted_pairs_summary AS (
                SELECT
                    asset_start, asset_end,
                    (dex_ex_pairs_summary.price - greatest(price_then, 0.000001)) / greatest(price_then, 0.000001) * 100 AS price_change,
                    liquidity * ed_end.price AS liquidity,
                    direct_volume_indexing_denom_over_window AS dv,
                    swap_volume_indexing_denom_over_window AS sv,
                    trades_over_window as trades
                FROM dex_ex_pairs_summary
                JOIN eligible_denoms AS ed_end
                ON ed_end.asset = asset_end
                JOIN eligible_denoms AS ed_start
                ON ed_start.asset = asset_start
                WHERE the_window = $3
            ),
            sums AS (
                SELECT
                    SUM(dv) AS direct_volume,
                    SUM(sv) AS swap_volume,
                    SUM(liquidity) AS liquidity,
                    SUM(trades) AS trades,
                    (SELECT COUNT(*) FROM converted_pairs_summary WHERE dv > 0 OR sv > 0) AS active_pairs
                FROM converted_pairs_summary
            ),
            largest_sv AS (
                SELECT
                    asset_start AS largest_sv_trading_pair_start,
                    asset_end AS largest_sv_trading_pair_end,
                    sv AS largest_sv_trading_pair_volume                    
                FROM converted_pairs_summary
                ORDER BY sv DESC
                LIMIT 1
            ),
            largest_dv AS (
                SELECT
                    asset_start AS largest_dv_trading_pair_start,
                    asset_end AS largest_dv_trading_pair_end,
                    dv AS largest_dv_trading_pair_volume                    
                FROM converted_pairs_summary
                ORDER BY dv DESC
                LIMIT 1
            ),
            top_price_mover AS (
                SELECT
                    asset_start AS top_price_mover_start,
                    asset_end AS top_price_mover_end,
                    price_change AS top_price_mover_change_percent
                FROM converted_pairs_summary
                ORDER BY price_change DESC
                LIMIT 1
            )
        INSERT INTO dex_ex_aggregate_summary
        SELECT
            $3,
            direct_volume, swap_volume, liquidity, trades, active_pairs,
            largest_sv_trading_pair_start,
            largest_sv_trading_pair_end,
            largest_sv_trading_pair_volume,
            largest_dv_trading_pair_start,
            largest_dv_trading_pair_end,
            largest_dv_trading_pair_volume,
            top_price_mover_start,
            top_price_mover_end,
            top_price_mover_change_percent
        FROM
            sums
        JOIN largest_sv ON TRUE
        JOIN largest_dv ON TRUE
        JOIN top_price_mover ON TRUE
        ON CONFLICT (the_window) DO UPDATE SET
            direct_volume = EXCLUDED.direct_volume,
            swap_volume = EXCLUDED.swap_volume,
            liquidity = EXCLUDED.liquidity,
            trades = EXCLUDED.trades,
            active_pairs = EXCLUDED.active_pairs,          
            largest_sv_trading_pair_start = EXCLUDED.largest_sv_trading_pair_start,
            largest_sv_trading_pair_end = EXCLUDED.largest_sv_trading_pair_end,
            largest_sv_trading_pair_volume = EXCLUDED.largest_sv_trading_pair_volume,
            largest_dv_trading_pair_start = EXCLUDED.largest_dv_trading_pair_start,
            largest_dv_trading_pair_end = EXCLUDED.largest_dv_trading_pair_end,
            largest_dv_trading_pair_volume = EXCLUDED.largest_dv_trading_pair_volume,
            top_price_mover_start = EXCLUDED.top_price_mover_start,
            top_price_mover_end = EXCLUDED.top_price_mover_end,
            top_price_mover_change_percent = EXCLUDED.top_price_mover_change_percent
        ")
        .bind(denom.to_bytes())
        .bind(min_liquidity)
        .bind(window.to_string())
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }
}

mod metadata {
    use super::*;

    pub async fn set(dbtx: &mut PgTransaction<'_>, quote_asset: asset::Id) -> anyhow::Result<()> {
        sqlx::query(
            "
        INSERT INTO dex_ex_metadata
        VALUES (1, $1)
        ON CONFLICT (id) DO UPDATE 
        SET id = EXCLUDED.id,
            quote_asset_id = EXCLUDED.quote_asset_id
        ",
        )
        .bind(quote_asset.to_bytes())
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct PairMetrics {
    trades: f64,
    liquidity_change: f64,
}

#[derive(Debug)]
struct Events {
    time: Option<DateTime>,
    height: i32,
    candles: HashMap<DirectedTradingPair, Candle>,
    metrics: HashMap<DirectedTradingPair, PairMetrics>,
    // Relevant positions.
    positions: BTreeMap<PositionId, Position>,
    // Store events
    position_opens: Vec<EventPositionOpen>,
    position_executions: Vec<EventPositionExecution>,
    position_closes: Vec<EventPositionClose>,
    position_withdrawals: Vec<EventPositionWithdraw>,
    batch_swaps: Vec<EventBatchSwap>,
    // Track transaction hashes by position ID
    position_open_txs: BTreeMap<PositionId, [u8; 32]>,
    position_close_txs: BTreeMap<PositionId, [u8; 32]>,
    position_withdrawal_txs: BTreeMap<PositionId, [u8; 32]>,
}

impl Events {
    fn new() -> Self {
        Self {
            time: None,
            height: 0,
            candles: HashMap::new(),
            metrics: HashMap::new(),
            positions: BTreeMap::new(),
            position_opens: Vec::new(),
            position_executions: Vec::new(),
            position_closes: Vec::new(),
            position_withdrawals: Vec::new(),
            batch_swaps: Vec::new(),
            position_open_txs: BTreeMap::new(),
            position_close_txs: BTreeMap::new(),
            position_withdrawal_txs: BTreeMap::new(),
        }
    }

    fn with_time(&mut self, time: DateTime) {
        self.time = Some(time)
    }

    fn with_candle(&mut self, pair: DirectedTradingPair, candle: Candle) {
        // Populate both this pair and the flipped pair, and if the flipped pair
        // is already populated, we need to mix the two candles together.
        let flip = pair.flip();
        let new_candle = match self.candles.get(&flip).cloned() {
            None => candle,
            Some(flipped) => {
                let mut out = candle;
                out.mix(&flipped);
                out
            }
        };

        self.candles.insert(pair, new_candle);
        self.candles.insert(flip, new_candle.flip());
    }

    fn metric(&mut self, pair: &DirectedTradingPair) -> &mut PairMetrics {
        if !self.metrics.contains_key(pair) {
            self.metrics.insert(*pair, PairMetrics::default());
        }
        // NOPANIC: inserted above.
        self.metrics.get_mut(pair).unwrap()
    }

    fn with_trade(&mut self, pair: &DirectedTradingPair) {
        self.metric(pair).trades += 1.0;
    }

    fn with_reserve_change(
        &mut self,
        pair: &TradingPair,
        old_reserves: Option<Reserves>,
        new_reserves: Reserves,
        removed: bool,
    ) {
        let (diff_1, diff_2) = match (removed, old_reserves, new_reserves) {
            (true, None, new) => (-(new.r1.value() as f64), -(new.r2.value() as f64)),
            (_, None, new) => ((new.r1.value() as f64), (new.r2.value() as f64)),
            (_, Some(old), new) => (
                (new.r1.value() as f64) - (old.r1.value() as f64),
                (new.r2.value() as f64) - (old.r2.value() as f64),
            ),
        };
        for (d_pair, diff) in [
            (
                DirectedTradingPair {
                    start: pair.asset_1(),
                    end: pair.asset_2(),
                },
                diff_2,
            ),
            (
                DirectedTradingPair {
                    start: pair.asset_2(),
                    end: pair.asset_1(),
                },
                diff_1,
            ),
        ] {
            self.metric(&d_pair).liquidity_change += diff;
        }
    }

    pub fn extract(block: &BlockEvents) -> anyhow::Result<Self> {
        let mut out = Self::new();
        out.height = block.height as i32;

        for event in &block.events {
            if let Ok(e) = EventCandlestickData::try_from_event(&event.event) {
                let candle = Candle::from_candlestick_data(&e.stick);
                out.with_candle(e.pair, candle);
            } else if let Ok(e) = EventBlockRoot::try_from_event(&event.event) {
                let time = DateTime::from_timestamp(e.timestamp_seconds, 0).ok_or(anyhow!(
                    "creating timestamp should succeed; timestamp: {}",
                    e.timestamp_seconds
                ))?;
                out.with_time(time);
            } else if let Ok(e) = EventPositionOpen::try_from_event(&event.event) {
                out.with_reserve_change(
                    &e.trading_pair,
                    None,
                    Reserves {
                        r1: e.reserves_1,
                        r2: e.reserves_2,
                    },
                    false,
                );
                if let Some(tx_hash) = event.tx_hash {
                    out.position_open_txs.insert(e.position_id, tx_hash);
                }
                // A newly opened position might be executed against in this block,
                // but wouldn't already be in the database. Adding it here ensures
                // it's available.
                out.positions.insert(e.position_id, e.position.clone());
                out.position_opens.push(e);
            } else if let Ok(e) = EventPositionWithdraw::try_from_event(&event.event) {
                // TODO: use close positions to track liquidity more precisely, in practic I (ck) expect few
                // positions to close with being withdrawn.
                out.with_reserve_change(
                    &e.trading_pair,
                    None,
                    Reserves {
                        r1: e.reserves_1,
                        r2: e.reserves_2,
                    },
                    true,
                );
                if let Some(tx_hash) = event.tx_hash {
                    out.position_withdrawal_txs.insert(e.position_id, tx_hash);
                }
                out.position_withdrawals.push(e);
            } else if let Ok(e) = EventPositionExecution::try_from_event(&event.event) {
                out.with_reserve_change(
                    &e.trading_pair,
                    Some(Reserves {
                        r1: e.prev_reserves_1,
                        r2: e.prev_reserves_2,
                    }),
                    Reserves {
                        r1: e.reserves_1,
                        r2: e.reserves_2,
                    },
                    false,
                );
                if e.reserves_1 > e.prev_reserves_1 {
                    // Whatever asset we ended up with more with was traded in.
                    out.with_trade(&DirectedTradingPair {
                        start: e.trading_pair.asset_1(),
                        end: e.trading_pair.asset_2(),
                    });
                } else if e.reserves_2 > e.prev_reserves_2 {
                    out.with_trade(&DirectedTradingPair {
                        start: e.trading_pair.asset_2(),
                        end: e.trading_pair.asset_1(),
                    });
                }
                out.position_executions.push(e);
            } else if let Ok(e) = EventPositionClose::try_from_event(&event.event) {
                out.position_closes.push(e);
            } else if let Ok(e) = EventQueuePositionClose::try_from_event(&event.event) {
                // The position close event is emitted by the dex module at EOB,
                // so we need to track it with the tx hash of the closure tx.
                if let Some(tx_hash) = event.tx_hash {
                    out.position_close_txs.insert(e.position_id, tx_hash);
                }
            } else if let Ok(e) = EventBatchSwap::try_from_event(&event.event) {
                out.batch_swaps.push(e);
            }
        }
        Ok(out)
    }

    async fn load_positions(&mut self, dbtx: &mut PgTransaction<'_>) -> anyhow::Result<()> {
        // Collect position IDs that we need but don't already have
        let missing_positions: Vec<_> = self
            .position_executions
            .iter()
            .map(|e| e.position_id)
            .filter(|id| !self.positions.contains_key(id))
            .collect();

        if missing_positions.is_empty() {
            return Ok(());
        }

        // Load missing positions from database
        let rows = sqlx::query(
            "SELECT position_raw 
             FROM dex_ex_position_state 
             WHERE position_id = ANY($1)",
        )
        .bind(
            &missing_positions
                .iter()
                .map(|id| id.0.as_ref())
                .collect::<Vec<_>>(),
        )
        .fetch_all(dbtx.as_mut())
        .await?;

        // Decode and store each position
        for row in rows {
            let position_raw: Vec<u8> = row.get("position_raw");
            let position = Position::decode(position_raw.as_slice())?;
            self.positions.insert(position.id(), position);
        }

        Ok(())
    }

    /// Attempt to find the price, relative to a given indexing denom, for a particular asset, in this block.
    pub fn price_for(&self, indexing_denom: asset::Id, asset: asset::Id) -> Option<f64> {
        self.candles
            .get(&DirectedTradingPair::new(asset, indexing_denom))
            .map(|x| x.close)
    }
}

#[derive(Debug)]
pub struct Component {
    denom: asset::Id,
    min_liquidity: f64,
}

impl Component {
    pub fn new(denom: asset::Id, min_liquidity: f64) -> Self {
        Self {
            denom,
            min_liquidity,
        }
    }

    async fn record_position_open(
        &self,
        dbtx: &mut PgTransaction<'_>,
        time: DateTime,
        height: i32,
        tx_hash: Option<[u8; 32]>,
        event: &EventPositionOpen,
    ) -> anyhow::Result<()> {
        // Get effective prices by orienting the trading function in each direction
        let effective_price_1_to_2: f64 = event
            .position
            .phi
            .orient_start(event.trading_pair.asset_1())
            .expect("position trading pair matches")
            .effective_price()
            .into();

        let effective_price_2_to_1: f64 = event
            .position
            .phi
            .orient_start(event.trading_pair.asset_2())
            .expect("position trading pair matches")
            .effective_price()
            .into();

        // First insert initial reserves and get the rowid
        let opening_reserves_rowid = sqlx::query_scalar::<_, i32>(
            "INSERT INTO dex_ex_position_reserves (
                position_id,
                height,
                time,
                reserves_1,
                reserves_2
            ) VALUES ($1, $2, $3, $4, $5) RETURNING rowid",
        )
        .bind(event.position_id.0)
        .bind(height)
        .bind(time)
        .bind(BigDecimal::from(event.reserves_1.value()))
        .bind(BigDecimal::from(event.reserves_2.value()))
        .fetch_one(dbtx.as_mut())
        .await?;

        // Then insert position state with the opening_reserves_rowid
        sqlx::query(
            "INSERT INTO dex_ex_position_state (
                position_id,
                asset_1,
                asset_2,
                p,
                q,
                close_on_fill,
                fee_bps,
                effective_price_1_to_2,
                effective_price_2_to_1,
                position_raw,
                opening_time,
                opening_height,
                opening_tx,
                opening_reserves_rowid
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        )
        .bind(event.position_id.0)
        .bind(event.trading_pair.asset_1().to_bytes())
        .bind(event.trading_pair.asset_2().to_bytes())
        .bind(BigDecimal::from(event.position.phi.component.p.value()))
        .bind(BigDecimal::from(event.position.phi.component.q.value()))
        .bind(event.position.close_on_fill)
        .bind(event.trading_fee as i32)
        .bind(effective_price_1_to_2)
        .bind(effective_price_2_to_1)
        .bind(event.position.encode_to_vec())
        .bind(time)
        .bind(height)
        .bind(tx_hash.map(|h| h.as_ref().to_vec()))
        .bind(opening_reserves_rowid)
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }

    async fn record_swap_execution_traces(
        &self,
        dbtx: &mut PgTransaction<'_>,
        time: DateTime,
        height: i32,
        swap_execution: &SwapExecution,
    ) -> anyhow::Result<()> {
        let SwapExecution {
            traces,
            input: se_input,
            output: se_output,
        } = swap_execution;

        let asset_start = se_input.asset_id;
        let asset_end = se_output.asset_id;
        let batch_input = se_input.amount;
        let batch_output = se_output.amount;

        for trace in traces.iter() {
            let Some(input_value) = trace.first() else {
                continue;
            };
            let Some(output_value) = trace.last() else {
                continue;
            };

            let input = input_value.amount;
            let output = output_value.amount;

            let price_float = (output.value() as f64) / (input.value() as f64);
            let amount_hops = trace
                .iter()
                .map(|x| BigDecimal::from(x.amount.value()))
                .collect::<Vec<_>>();
            let position_id_hops: Vec<[u8; 32]> = vec![];
            let asset_hops = trace
                .iter()
                .map(|x| x.asset_id.to_bytes())
                .collect::<Vec<_>>();

            sqlx::query(
                "INSERT INTO dex_ex_batch_swap_traces (
                height,
                time,
                input,
                output,
                batch_input,
                batch_output,
                price_float,
                asset_start,
                asset_end,
                asset_hops,
                amount_hops,
               position_id_hops
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
            )
            .bind(height)
            .bind(time)
            .bind(BigDecimal::from(input.value()))
            .bind(BigDecimal::from(output.value()))
            .bind(BigDecimal::from(batch_input.value()))
            .bind(BigDecimal::from(batch_output.value()))
            .bind(price_float)
            .bind(asset_start.to_bytes())
            .bind(asset_end.to_bytes())
            .bind(asset_hops)
            .bind(amount_hops)
            .bind(position_id_hops)
            .execute(dbtx.as_mut())
            .await?;
        }

        Ok(())
    }

    async fn record_batch_swap_traces(
        &self,
        dbtx: &mut PgTransaction<'_>,
        time: DateTime,
        height: i32,
        event: &EventBatchSwap,
    ) -> anyhow::Result<()> {
        let EventBatchSwap {
            batch_swap_output_data: _,
            swap_execution_1_for_2,
            swap_execution_2_for_1,
        } = event;

        if let Some(batch_swap_1_2) = swap_execution_1_for_2 {
            self.record_swap_execution_traces(dbtx, time, height, batch_swap_1_2)
                .await?;
        }

        if let Some(batch_swap_2_1) = swap_execution_2_for_1 {
            self.record_swap_execution_traces(dbtx, time, height, batch_swap_2_1)
                .await?;
        }

        Ok(())
    }

    async fn record_position_execution(
        &self,
        dbtx: &mut PgTransaction<'_>,
        time: DateTime,
        height: i32,
        event: &EventPositionExecution,
        positions: &BTreeMap<PositionId, Position>,
    ) -> anyhow::Result<()> {
        // Get the position that was executed against
        let position = positions
            .get(&event.position_id)
            .expect("position must exist for execution");

        // Determine trade direction and compute deltas
        let (delta_1, delta_2, lambda_1, lambda_2) = if event.reserves_1 > event.prev_reserves_1 {
            // Asset 1 was input
            let delta_1 = event.reserves_1 - event.prev_reserves_1;
            let lambda_2 = event.prev_reserves_2 - event.reserves_2;
            (delta_1, Amount::zero(), Amount::zero(), lambda_2)
        } else {
            // Asset 2 was input
            let delta_2 = event.reserves_2 - event.prev_reserves_2;
            let lambda_1 = event.prev_reserves_1 - event.reserves_1;
            (Amount::zero(), delta_2, lambda_1, Amount::zero())
        };

        // Compute fees directly from input amounts using u128 arithmetic
        let fee_bps = position.phi.component.fee as u128;
        let fee_1 = (delta_1.value() * fee_bps) / 10_000u128;
        let fee_2 = (delta_2.value() * fee_bps) / 10_000u128;

        // First insert the reserves and get the rowid
        let reserves_rowid = sqlx::query_scalar::<_, i32>(
            "INSERT INTO dex_ex_position_reserves (
                position_id,
                height,
                time,
                reserves_1,
                reserves_2
            ) VALUES ($1, $2, $3, $4, $5) RETURNING rowid",
        )
        .bind(event.position_id.0)
        .bind(height)
        .bind(time)
        .bind(BigDecimal::from(event.reserves_1.value()))
        .bind(BigDecimal::from(event.reserves_2.value()))
        .fetch_one(dbtx.as_mut())
        .await?;

        // Then record the execution with the reserves_rowid
        sqlx::query(
            "INSERT INTO dex_ex_position_executions (
                position_id,
                height,
                time,
                reserves_rowid,
                delta_1,
                delta_2,
                lambda_1,
                lambda_2,
                fee_1,
                fee_2,
                context_asset_start,
                context_asset_end
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(event.position_id.0)
        .bind(height)
        .bind(time)
        .bind(reserves_rowid)
        .bind(BigDecimal::from(delta_1.value()))
        .bind(BigDecimal::from(delta_2.value()))
        .bind(BigDecimal::from(lambda_1.value()))
        .bind(BigDecimal::from(lambda_2.value()))
        .bind(BigDecimal::from(fee_1))
        .bind(BigDecimal::from(fee_2))
        .bind(event.context.start.to_bytes())
        .bind(event.context.end.to_bytes())
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }

    async fn record_position_close(
        &self,
        dbtx: &mut PgTransaction<'_>,
        time: DateTime,
        height: i32,
        tx_hash: Option<[u8; 32]>,
        event: &EventPositionClose,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE dex_ex_position_state 
             SET closing_time = $1,
                 closing_height = $2,
                 closing_tx = $3
             WHERE position_id = $4",
        )
        .bind(time)
        .bind(height)
        .bind(tx_hash.map(|h| h.as_ref().to_vec()))
        .bind(event.position_id.0)
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }

    async fn record_position_withdraw(
        &self,
        dbtx: &mut PgTransaction<'_>,
        time: DateTime,
        height: i32,
        tx_hash: Option<[u8; 32]>,
        event: &EventPositionWithdraw,
    ) -> anyhow::Result<()> {
        // First insert the final reserves state (zeros after withdrawal)
        let reserves_rowid = sqlx::query_scalar::<_, i32>(
            "INSERT INTO dex_ex_position_reserves (
                position_id,
                height,
                time,
                reserves_1,
                reserves_2
            ) VALUES ($1, $2, $3, $4, $4) RETURNING rowid", // Using $4 twice for zero values
        )
        .bind(event.position_id.0)
        .bind(height)
        .bind(time)
        .bind(BigDecimal::from(0)) // Both reserves become zero after withdrawal
        .fetch_one(dbtx.as_mut())
        .await?;

        sqlx::query(
            "INSERT INTO dex_ex_position_withdrawals (
                position_id,
                height,
                time,
                withdrawal_tx,
                sequence,
                reserves_1,
                reserves_2,
                reserves_rowid
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(event.position_id.0)
        .bind(height)
        .bind(time)
        .bind(tx_hash.map(|h| h.as_ref().to_vec()))
        .bind(event.sequence as i32)
        .bind(BigDecimal::from(event.reserves_1.value()))
        .bind(BigDecimal::from(event.reserves_2.value()))
        .bind(reserves_rowid)
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
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

    fn name(&self) -> String {
        "dex_ex".to_string()
    }

    async fn index_batch(
        &self,
        dbtx: &mut PgTransaction,
        batch: EventBatch,
    ) -> Result<(), anyhow::Error> {
        metadata::set(dbtx, self.denom).await?;
        let mut charts = HashMap::new();
        let mut snapshots = HashMap::new();
        let mut last_time = None;
        for block in batch.by_height.iter() {
            let mut events = Events::extract(&block)?;
            let time = events
                .time
                .expect(&format!("no block root event at height {}", block.height));
            last_time = Some(time);

            // Load any missing positions before processing events
            events.load_positions(dbtx).await?;

            // Record batch swap execution traces.
            for event in &events.batch_swaps {
                self.record_batch_swap_traces(dbtx, time, block.height as i32, event)
                    .await?;
            }

            // Record position opens
            for event in &events.position_opens {
                let tx_hash = events.position_open_txs.get(&event.position_id).copied();
                self.record_position_open(dbtx, time, events.height, tx_hash, event)
                    .await?;
            }

            // Process position executions
            for event in &events.position_executions {
                self.record_position_execution(dbtx, time, events.height, event, &events.positions)
                    .await?;
            }

            // Record position closes
            for event in &events.position_closes {
                let tx_hash = events.position_close_txs.get(&event.position_id).copied();
                self.record_position_close(dbtx, time, events.height, tx_hash, event)
                    .await?;
            }

            // Record position withdrawals
            for event in &events.position_withdrawals {
                let tx_hash = events
                    .position_withdrawal_txs
                    .get(&event.position_id)
                    .copied();
                self.record_position_withdraw(dbtx, time, events.height, tx_hash, event)
                    .await?;
            }

            for (pair, candle) in &events.candles {
                for window in Window::all() {
                    let key = (pair.start, pair.end, window);
                    if !charts.contains_key(&key) {
                        let ctx = PriceChartContext::load(dbtx, key.0, key.1, key.2).await?;
                        charts.insert(key, ctx);
                    }
                    charts
                        .get_mut(&key)
                        .unwrap() // safe because we just inserted above
                        .update(dbtx, time, *candle)
                        .await?;
                }
            }

            let block_pairs = events
                .candles
                .keys()
                .chain(events.metrics.keys())
                .copied()
                .collect::<HashSet<_>>();
            for pair in block_pairs {
                if !snapshots.contains_key(&pair) {
                    let ctx = summary::Context::load(dbtx, pair.start, pair.end).await?;
                    snapshots.insert(pair, ctx);
                }
                // NOPANIC: inserted above
                snapshots
                    .get_mut(&pair)
                    .unwrap()
                    .update(
                        dbtx,
                        time,
                        events.candles.get(&pair).copied(),
                        events.metrics.get(&pair).copied().unwrap_or_default(),
                        events.price_for(self.denom, pair.start),
                    )
                    .await?;
            }
        }

        if let Some(now) = last_time {
            for window in Window::all() {
                for pair in snapshots.keys() {
                    summary::update_summary(dbtx, now, pair.start, pair.end, window).await?;
                }
                summary::update_aggregate_summary(dbtx, window, self.denom, self.min_liquidity)
                    .await?;
            }
        }
        for chart in charts.into_values() {
            chart.unload(dbtx).await?;
        }
        Ok(())
    }
}
