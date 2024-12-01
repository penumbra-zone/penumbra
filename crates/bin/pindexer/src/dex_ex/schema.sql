-- Contains, for each directed asset pair and window type, candle sticks for each window.
CREATE TABLE IF NOT EXISTS dex_ex_price_charts (
  -- We just want a simple primary key to have here.
  id SERIAL PRIMARY KEY,
  -- The bytes for the first asset in the directed pair.
  asset_start BYTEA NOT NULL,
  -- The bytes for the second asset in the directed pair.
  asset_end BYTEA NOT NULL,
  -- The window type for this stick.
  --
  -- Enum types are annoying.
  the_window TEXT NOT NULL,
  -- The start time of this window.
  start_time TIMESTAMPTZ NOT NULL,
  -- The price at the start of a window.
  open FLOAT8 NOT NULL,
  -- The price at the close of a window.
  close FLOAT8 NOT NULL,
  -- The highest price reached during a window.
  high FLOAT8 NOT NULL,
  -- The lowest price reached during a window.
  low FLOAT8 NOT NULL,
  -- The volume traded directly through position executions.
  direct_volume FLOAT8 NOT NULL,
  -- The volume that traded indirectly, possibly through several positions.
  swap_volume FLOAT8 NOT NULL
);

CREATE UNIQUE INDEX ON dex_ex_price_charts (asset_start, asset_end, the_window, start_time);

CREATE TABLE IF NOT EXISTS dex_ex_pairs_block_snapshot (
  id SERIAL PRIMARY KEY,
  time TIMESTAMPTZ NOT NULL,
  asset_start BYTEA NOT NULL,
  asset_end BYTEA NOT NULL,
  price FLOAT8 NOT NULL,
  liquidity FLOAT8 NOT NULL,
  direct_volume FLOAT8 NOT NULL,
  swap_volume FLOAT8 NOT NULL,
  trades FLOAT8 NOT NULL
);

CREATE UNIQUE INDEX ON dex_ex_pairs_block_snapshot (time, asset_start, asset_end);
CREATE INDEX ON dex_ex_pairs_block_snapshot (asset_start, asset_end);

CREATE TABLE IF NOT EXISTS dex_ex_pairs_summary (
  asset_start BYTEA NOT NULL,
  asset_end BYTEA NOT NULL,
  the_window TEXT NOT NULL,
  price FLOAT8 NOT NULL,
  price_then FLOAT8 NOT NULL,
  low FLOAT8 NOT NULL,
  high FLOAT8 NOT NULL,
  liquidity FLOAT8 NOT NULL,
  liquidity_then FLOAT8 NOT NULL,
  direct_volume_over_window FLOAT8 NOT NULL,
  swap_volume_over_window FLOAT8 NOT NULL,
  trades_over_window FLOAT8 NOT NULL,
  PRIMARY KEY (asset_start, asset_end, the_window)
);

CREATE TABLE IF NOT EXISTS dex_ex_aggregate_summary (
  the_window TEXT PRIMARY KEY,
  direct_volume FLOAT8 NOT NULL,
  swap_volume FLOAT8 NOT NULL,
  liquidity FLOAT8 NOT NULL,
  trades FLOAT8 NOT NULL,
  active_pairs FLOAT8 NOT NULL,
  largest_sv_trading_pair_start BYTEA NOT NULL,
  largest_sv_trading_pair_end BYTEA NOT NULL,
  largest_sv_trading_pair_volume FLOAT8 NOT NULL,
  largest_dv_trading_pair_start BYTEA NOT NULL,
  largest_dv_trading_pair_end BYTEA NOT NULL,
  largest_dv_trading_pair_volume FLOAT8 NOT NULL,
  top_price_mover_start BYTEA NOT NULL,
  top_price_mover_end BYTEA NOT NULL,
  top_price_mover_change_percent FLOAT8 NOT NULL
);

CREATE TABLE IF NOT EXISTS dex_ex_metadata (
  id INT PRIMARY KEY,
  -- The asset id to use for prices in places such as the aggregate summary.
  quote_asset_id BYTEA NOT NULL
);
