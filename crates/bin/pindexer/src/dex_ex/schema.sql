CREATE TABLE IF NOT EXISTS dex_ex_candlestick (
  id SERIAL PRIMARY KEY,
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

DROP TYPE IF EXISTS dex_ex_Window CASCADE;
CREATE TYPE dex_ex_Window as ENUM (
  '1m',
  '15m',
  '1h',
  '4h',
  '1d',
  '1w',
  '1mo'
);

-- Contains, for each directed asset pair and window type, candle sticks for each window.
CREATE TABLE IF NOT EXISTS dex_ex_price_charts (
  -- We just want a simple primary key to have here.
  id SERIAL PRIMARY KEY,
  -- The bytes for the first asset in the directed pair.
  asset_id_1 BYTEA NOT NULL,
  -- The bytes for the second asset in the directed pair.
  asset_id_2 BYTEA NOT NULL,
  -- The window type for this stick.
  the_window dex_ex_Window NOT NULL,
  -- The start time of this window.
  start_time TIMESTAMPTZ NOT NULL,
  -- The start time for the window this stick is about.
  candlestick_id INTEGER REFERENCES dex_ex_candlestick (id)
);

CREATE UNIQUE INDEX ON dex_ex_price_charts (asset_id_1, asset_id_2, the_window, start_time);

CREATE TABLE IF NOT EXISTS dex_ex_summary (
  -- The first asset of the directed pair.
  asset_id_1 BYTEA NOT NULL,
  -- The second asset of the directed pair.
  asset_id_2 BYTEA NOT NULL,
  -- The current price (in terms of asset2)
  current_price FLOAT8 NOT NULL,
  -- The price 24h ago.
  price_24h_ago FLOAT8 NOT NULL,
  -- The highest price over the past 24h.
  high_24h FLOAT8 NOT NULL,
  -- The lowest price over the past 24h.
  low_24h FLOAT8 NOT NULL,
  -- c.f. candlesticks for the difference between these two
  direct_volume_24h FLOAT8 NOT NULL,
  swap_volume_24h FLOAT8 NOT NULL,
  PRIMARY KEY (asset_id_1, asset_id_2)
);
