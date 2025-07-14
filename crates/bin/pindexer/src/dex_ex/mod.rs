use anyhow::anyhow;
use clap::Result;
use cometindex::{
    async_trait,
    index::{BlockEvents, EventBatch, EventBatchContext, Version},
    AppView, PgTransaction,
};
use penumbra_sdk_asset::{asset, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_dex::{
    event::{
        EventArbExecution, EventBatchSwap, EventPositionClose, EventPositionExecution,
        EventPositionOpen, EventPositionWithdraw, EventQueuePositionClose,
    },
    lp::{position::Flows, Reserves},
    DirectedTradingPair, SwapExecution, TradingPair,
};
use penumbra_sdk_dex::{
    event::{EventSwap, EventSwapClaim},
    lp::position::{Id as PositionId, Position},
};
use penumbra_sdk_funding::event::EventLqtPositionReward;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::event::EventDomainType;
use penumbra_sdk_proto::DomainType;
use penumbra_sdk_sct::event::EventBlockRoot;
use penumbra_sdk_transaction::Transaction;
use sqlx::types::BigDecimal;
use sqlx::Row;
use std::collections::{BTreeMap, HashMap, HashSet};

type DateTime = sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>;

mod candle {
    use super::DateTime;
    use chrono::{Datelike as _, Days, TimeDelta, TimeZone as _, Timelike as _, Utc};
    use penumbra_sdk_dex::CandlestickData;
    use std::fmt::Display;

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

        pub fn point(price: f64, direct_volume: f64) -> Self {
            Self {
                open: price,
                close: price,
                low: price,
                high: price,
                direct_volume,
                swap_volume: 0.0,
            }
        }

        pub fn merge(&mut self, that: &Self) {
            self.close = that.close;
            self.low = self.low.min(that.low);
            self.high = self.high.max(that.high);
            self.direct_volume += that.direct_volume;
            self.swap_volume += that.swap_volume;
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
                |(open, close, high, low, direct_volume, swap_volume, start)| {
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

    pub async fn update_summaries(
        dbtx: &mut PgTransaction<'_>,
        now: DateTime,
        window: Window,
    ) -> anyhow::Result<()> {
        let then = window.subtract_from(now);
        let pairs = sqlx::query_as::<_, (Vec<u8>, Vec<u8>)>(
            "SELECT asset_start, asset_end FROM dex_ex_pairs_block_snapshot GROUP BY asset_start, asset_end",
        )
        .fetch_all(dbtx.as_mut())
        .await?;
        for (start, end) in pairs {
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
                WHERE time <= $3
                ORDER BY time DESC
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
            .bind(start)
            .bind(end)
            .bind(now)
            .bind(then)
            .bind(window.to_string())
            .execute(dbtx.as_mut())
            .await?;
        }
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
                AND liquidity >= $2
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
                    SUM(trades) AS trades
                FROM converted_pairs_summary WHERE asset_start < asset_end
            ),
            total_liquidity AS (
                SELECT
                    SUM(liquidity) AS liquidity
                FROM converted_pairs_summary
            ),
            undirected_pairs_summary AS (
              WITH pairs AS (
                  SELECT asset_start AS a_start, asset_end AS a_end FROM converted_pairs_summary WHERE asset_start < asset_end
              )
              SELECT
                  a_start AS asset_start,
                  a_end AS asset_end,
                  (SELECT SUM(sv) FROM converted_pairs_summary WHERE (asset_start = a_start AND asset_end = a_end) OR (asset_start = a_end AND asset_end = a_start)) AS sv,
                  (SELECT SUM(dv) FROM converted_pairs_summary WHERE (asset_start = a_start AND asset_end = a_end) OR (asset_start = a_end AND asset_end = a_start)) AS dv
              FROM pairs
            ),
            counts AS (
              SELECT COUNT(*) AS active_pairs FROM undirected_pairs_summary WHERE dv > 0
            ),
            largest_sv AS (
                SELECT
                    asset_start AS largest_sv_trading_pair_start,
                    asset_end AS largest_sv_trading_pair_end,
                    sv AS largest_sv_trading_pair_volume                    
                FROM undirected_pairs_summary
                ORDER BY sv DESC
                LIMIT 1
            ),
            largest_dv AS (
                SELECT
                    asset_start AS largest_dv_trading_pair_start,
                    asset_end AS largest_dv_trading_pair_end,
                    dv AS largest_dv_trading_pair_volume                    
                FROM undirected_pairs_summary
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
        JOIN total_liquidity ON TRUE
        JOIN counts ON TRUE
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

#[derive(Debug, Clone, serde::Serialize)]
struct BatchSwapSummary {
    asset_start: asset::Id,
    asset_end: asset::Id,
    input: Amount,
    output: Amount,
    num_swaps: i32,
    price_float: f64,
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
    swaps: BTreeMap<TradingPair, Vec<EventSwap>>,
    swap_claims: BTreeMap<TradingPair, Vec<EventSwapClaim>>,
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
            swaps: BTreeMap::new(),
            swap_claims: BTreeMap::new(),
            position_open_txs: BTreeMap::new(),
            position_close_txs: BTreeMap::new(),
            position_withdrawal_txs: BTreeMap::new(),
        }
    }

    fn with_time(&mut self, time: DateTime) {
        self.time = Some(time)
    }

    fn metric(&mut self, pair: &DirectedTradingPair) -> &mut PairMetrics {
        if !self.metrics.contains_key(pair) {
            self.metrics.insert(*pair, PairMetrics::default());
        }
        // NOPANIC: inserted above.
        self.metrics.get_mut(pair).unwrap()
    }

    fn with_trace(&mut self, input: Value, output: Value) {
        let pair_1_2 = DirectedTradingPair {
            start: input.asset_id,
            end: output.asset_id,
        };
        // This shouldn't happen, but to avoid weird stuff later, let's ignore this trace.
        if pair_1_2.start == pair_1_2.end {
            tracing::warn!(?input, ?output, "found trace with a loop?");
            return;
        }
        let pair_2_1 = pair_1_2.flip();
        let input_amount = f64::from(input.amount);
        let output_amount = f64::from(output.amount);
        if input_amount == 0.0 && output_amount == 0.0 {
            tracing::warn!(?input, ?output, "ignoring trace with 0 input and output");
            return;
        }
        let price_1_2 = output_amount / input_amount;
        let candle_1_2 = Candle::point(price_1_2, input_amount);
        let candle_2_1 = Candle::point(1.0 / price_1_2, output_amount);
        self.metric(&pair_1_2).trades += 1.0;
        self.candles
            .entry(pair_1_2)
            .and_modify(|c| c.merge(&candle_1_2))
            .or_insert(candle_1_2);
        self.metric(&pair_2_1).trades += 1.0;
        self.candles
            .entry(pair_2_1)
            .and_modify(|c| c.merge(&candle_2_1))
            .or_insert(candle_2_1);
    }

    fn with_swap_execution(&mut self, se: &SwapExecution) {
        for row in se.traces.iter() {
            for window in row.windows(2) {
                self.with_trace(window[0], window[1]);
            }
        }
        let pair_1_2 = DirectedTradingPair {
            start: se.input.asset_id,
            end: se.output.asset_id,
        };
        // When doing arb, we don't want to report the volume on UM -> UM,
        // so we need this check.
        if pair_1_2.start == pair_1_2.end {
            return;
        }
        let pair_2_1 = pair_1_2.flip();
        self.candles
            .entry(pair_1_2)
            .and_modify(|c| c.swap_volume += f64::from(se.input.amount));
        self.candles
            .entry(pair_2_1)
            .and_modify(|c| c.swap_volume += f64::from(se.output.amount));
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

    pub fn extract(block: &BlockEvents, ignore_arb_executions: bool) -> anyhow::Result<Self> {
        let mut out = Self::new();
        out.height = block.height() as i32;

        for event in block.events() {
            if let Ok(e) = EventBlockRoot::try_from_event(&event.event) {
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
                if let Some(tx_hash) = event.tx_hash() {
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
                if let Some(tx_hash) = event.tx_hash() {
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
                out.position_executions.push(e);
            } else if let Ok(e) = EventPositionClose::try_from_event(&event.event) {
                out.position_closes.push(e);
            } else if let Ok(e) = EventQueuePositionClose::try_from_event(&event.event) {
                // The position close event is emitted by the dex module at EOB,
                // so we need to track it with the tx hash of the closure tx.
                if let Some(tx_hash) = event.tx_hash() {
                    out.position_close_txs.insert(e.position_id, tx_hash);
                }
            } else if let Ok(e) = EventSwap::try_from_event(&event.event) {
                out.swaps
                    .entry(e.trading_pair)
                    .or_insert_with(Vec::new)
                    .push(e);
            } else if let Ok(e) = EventSwapClaim::try_from_event(&event.event) {
                out.swap_claims
                    .entry(e.trading_pair)
                    .or_insert_with(Vec::new)
                    .push(e);
            } else if let Ok(e) = EventLqtPositionReward::try_from_event(&event.event) {
                let pair = DirectedTradingPair {
                    start: e.incentivized_asset_id,
                    end: *STAKING_TOKEN_ASSET_ID,
                };
                out.metric(&pair).liquidity_change += e.reward_amount.value() as f64;
            } else if let Ok(e) = EventBatchSwap::try_from_event(&event.event) {
                // NOTE: order matters here, 2 for 1 happened after.
                if let Some(se) = e.swap_execution_1_for_2.as_ref() {
                    out.with_swap_execution(se);
                }
                if let Some(se) = e.swap_execution_2_for_1.as_ref() {
                    out.with_swap_execution(se);
                }
                out.batch_swaps.push(e);
            } else if let Ok(e) = EventArbExecution::try_from_event(&event.event) {
                if !ignore_arb_executions {
                    out.with_swap_execution(&e.swap_execution);
                }
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
    ignore_arb_executions: bool,
}

impl Component {
    pub fn new(denom: asset::Id, min_liquidity: f64, ignore_arb_executions: bool) -> Self {
        Self {
            denom,
            min_liquidity,
            ignore_arb_executions,
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

    async fn record_block_summary(
        &self,
        dbtx: &mut PgTransaction<'_>,
        time: DateTime,
        height: i32,
        events: &Events,
    ) -> anyhow::Result<()> {
        let num_opened_lps = events.position_opens.len() as i32;
        let num_closed_lps = events.position_closes.len() as i32;
        let num_withdrawn_lps = events.position_withdrawals.len() as i32;
        let num_swaps = events.swaps.iter().map(|(_, v)| v.len()).sum::<usize>() as i32;
        let num_swap_claims = events
            .swap_claims
            .iter()
            .map(|(_, v)| v.len())
            .sum::<usize>() as i32;
        let num_txs = events.batch_swaps.len() as i32;

        let mut batch_swap_summaries = Vec::<BatchSwapSummary>::new();

        for event in &events.batch_swaps {
            let trading_pair = event.batch_swap_output_data.trading_pair;

            if let Some(swap_1_2) = &event.swap_execution_1_for_2 {
                let asset_start = swap_1_2.input.asset_id;
                let asset_end = swap_1_2.output.asset_id;
                let input = swap_1_2.input.amount;
                let output = swap_1_2.output.amount;
                let price_float = (output.value() as f64) / (input.value() as f64);

                let empty_vec = vec![];
                let swaps_for_pair = events.swaps.get(&trading_pair).unwrap_or(&empty_vec);
                let filtered_swaps: Vec<_> = swaps_for_pair
                    .iter()
                    .filter(|swap| swap.delta_1_i != Amount::zero())
                    .collect::<Vec<_>>();
                let num_swaps = filtered_swaps.len() as i32;

                batch_swap_summaries.push(BatchSwapSummary {
                    asset_start,
                    asset_end,
                    input,
                    output,
                    num_swaps,
                    price_float,
                });
            }

            if let Some(swap_2_1) = &event.swap_execution_2_for_1 {
                let asset_start = swap_2_1.input.asset_id;
                let asset_end = swap_2_1.output.asset_id;
                let input = swap_2_1.input.amount;
                let output = swap_2_1.output.amount;
                let price_float = (output.value() as f64) / (input.value() as f64);

                let empty_vec = vec![];
                let swaps_for_pair = events.swaps.get(&trading_pair).unwrap_or(&empty_vec);
                let filtered_swaps: Vec<_> = swaps_for_pair
                    .iter()
                    .filter(|swap| swap.delta_2_i != Amount::zero())
                    .collect::<Vec<_>>();
                let num_swaps = filtered_swaps.len() as i32;

                batch_swap_summaries.push(BatchSwapSummary {
                    asset_start,
                    asset_end,
                    input,
                    output,
                    num_swaps,
                    price_float,
                });
            }
        }

        sqlx::query(
            "INSERT INTO dex_ex_block_summary (
            height,
            time,
            batch_swaps,
            num_open_lps,
            num_closed_lps,
            num_withdrawn_lps,
            num_swaps,
            num_swap_claims,
            num_txs
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(height)
        .bind(time)
        .bind(serde_json::to_value(&batch_swap_summaries)?)
        .bind(num_opened_lps)
        .bind(num_closed_lps)
        .bind(num_withdrawn_lps)
        .bind(num_swaps)
        .bind(num_swap_claims)
        .bind(num_txs)
        .execute(dbtx.as_mut())
        .await?;

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
        let current = Reserves {
            r1: event.reserves_1,
            r2: event.reserves_2,
        };
        let prev = Reserves {
            r1: event.prev_reserves_1,
            r2: event.prev_reserves_2,
        };
        let flows = Flows::from_phi_and_reserves(&position.phi, &current, &prev);

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
        .bind(BigDecimal::from(flows.delta_1().value()))
        .bind(BigDecimal::from(flows.delta_2().value()))
        .bind(BigDecimal::from(flows.lambda_1().value()))
        .bind(BigDecimal::from(flows.lambda_2().value()))
        .bind(BigDecimal::from(flows.fee_1().value()))
        .bind(BigDecimal::from(flows.fee_2().value()))
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

    async fn record_transaction(
        &self,
        dbtx: &mut PgTransaction<'_>,
        time: DateTime,
        height: u64,
        transaction_id: [u8; 32],
        transaction: Transaction,
    ) -> anyhow::Result<()> {
        if transaction.transaction_body.actions.is_empty() {
            return Ok(());
        }
        sqlx::query(
            "INSERT INTO dex_ex_transactions (
                transaction_id,
                transaction,
                height,
                time
            ) VALUES ($1, $2, $3, $4)
            ",
        )
        .bind(transaction_id)
        .bind(transaction.encode_to_vec())
        .bind(i32::try_from(height)?)
        .bind(time)
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }

    async fn record_all_transactions(
        &self,
        dbtx: &mut PgTransaction<'_>,
        time: DateTime,
        block: &BlockEvents,
    ) -> anyhow::Result<()> {
        for (tx_id, tx_bytes) in block.transactions() {
            let tx = Transaction::try_from(tx_bytes)?;
            let height = block.height();
            self.record_transaction(dbtx, time, height, tx_id, tx)
                .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl AppView for Component {
    async fn init_chain(
        &self,
        _dbtx: &mut PgTransaction,
        _: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn name(&self) -> String {
        "dex_ex".to_string()
    }

    fn version(&self) -> Version {
        let hash: [u8; 32] = blake2b_simd::Params::default()
            .personal(b"option_hash")
            .hash_length(32)
            .to_state()
            .update(&self.denom.to_bytes())
            .update(&self.min_liquidity.to_le_bytes())
            .update(&[u8::from(self.ignore_arb_executions)])
            .finalize()
            .as_bytes()
            .try_into()
            .expect("Impossible 000-001: expected 32 byte hash");
        Version::with_major(2).with_option_hash(hash)
    }

    async fn reset(&self, dbtx: &mut PgTransaction) -> Result<(), anyhow::Error> {
        for statement in include_str!("reset.sql").split(";") {
            sqlx::query(statement).execute(dbtx.as_mut()).await?;
        }
        Ok(())
    }

    async fn on_startup(&self, dbtx: &mut PgTransaction) -> Result<(), anyhow::Error> {
        for statement in include_str!("schema.sql").split(";") {
            sqlx::query(statement).execute(dbtx.as_mut()).await?;
        }
        Ok(())
    }

    async fn index_batch(
        &self,
        dbtx: &mut PgTransaction,
        batch: EventBatch,
        ctx: EventBatchContext,
    ) -> Result<(), anyhow::Error> {
        metadata::set(dbtx, self.denom).await?;
        let mut charts = HashMap::new();
        let mut snapshots = HashMap::new();
        let mut last_time = None;
        for block in batch.events_by_block() {
            let mut events = Events::extract(&block, self.ignore_arb_executions)?;
            let time = events
                .time
                .expect(&format!("no block root event at height {}", block.height()));
            last_time = Some(time);

            self.record_all_transactions(dbtx, time, block).await?;

            // Load any missing positions before processing events
            events.load_positions(dbtx).await?;

            // This is where we are going to build the block summary for the DEX.
            self.record_block_summary(dbtx, time, block.height() as i32, &events)
                .await?;

            // Record batch swap execution traces.
            for event in &events.batch_swaps {
                self.record_batch_swap_traces(dbtx, time, block.height() as i32, event)
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
                sqlx::query(
                    "INSERT INTO dex_ex.candles VALUES (DEFAULT, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                )
                .bind(i64::try_from(block.height())?)
                .bind(time)
                .bind(pair.start.to_bytes())
                .bind(pair.end.to_bytes())
                .bind(candle.open)
                .bind(candle.close)
                .bind(candle.low)
                .bind(candle.high)
                .bind(candle.direct_volume)
                .bind(candle.swap_volume)
                .execute(dbtx.as_mut())
                .await?;
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

        if ctx.is_last() {
            if let Some(now) = last_time {
                for window in Window::all() {
                    summary::update_summaries(dbtx, now, window).await?;
                    summary::update_aggregate_summary(dbtx, window, self.denom, self.min_liquidity)
                        .await?;
                }
            }
        }
        for chart in charts.into_values() {
            chart.unload(dbtx).await?;
        }
        Ok(())
    }
}
