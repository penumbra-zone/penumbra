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
  -- The most recent price of the start asset, in terms of the indexing denom.
  start_price_indexing_denom FLOAT8 NOT NULL,
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
  direct_volume_indexing_denom_over_window FLOAT8 NOT NULL,
  swap_volume_indexing_denom_over_window FLOAT8 NOT NULL,
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

CREATE TABLE IF NOT EXISTS dex_ex_position_state (
  -- Call this rowid to distinguish it from the position ID.
  rowid SERIAL PRIMARY KEY,
  -- Immutable position data, defining the trading function.
  position_id BYTEA NOT NULL UNIQUE,
  asset_1 BYTEA NOT NULL,
  asset_2 BYTEA NOT NULL,
  p NUMERIC(39) NOT NULL,
  q NUMERIC(39) NOT NULL,
  close_on_fill BOOLEAN NOT NULL,
  fee_bps INTEGER NOT NULL,
  effective_price_1_to_2 FLOAT8 NOT NULL,
  effective_price_2_to_1 FLOAT8 NOT NULL,
  position_raw BYTEA NOT NULL,
  -- The time and height at which the position was opened, and its initial reserves.
  opening_time TIMESTAMPTZ NOT NULL,
  opening_height INTEGER NOT NULL,
  opening_tx BYTEA,
  opening_reserves_rowid INTEGER NOT NULL,
  -- The time and height at which the position was closed, if it was closed.
  closing_time TIMESTAMPTZ,
  closing_height INTEGER,
  closing_tx BYTEA
);

CREATE INDEX ON dex_ex_position_state (position_id);
CREATE INDEX ON dex_ex_position_state (opening_tx);

CREATE TABLE IF NOT EXISTS dex_ex_position_reserves (
  rowid SERIAL PRIMARY KEY,
  position_id BYTEA NOT NULL,
  height INTEGER NOT NULL,
  time TIMESTAMPTZ NOT NULL,
  reserves_1 NUMERIC(39) NOT NULL,
  reserves_2 NUMERIC(39) NOT NULL
);

CREATE INDEX ON dex_ex_position_reserves (position_id, height, rowid);

CREATE TABLE IF NOT EXISTS dex_ex_position_executions (
  rowid SERIAL PRIMARY KEY,
  position_id BYTEA NOT NULL,
  height INTEGER NOT NULL,
  time TIMESTAMPTZ NOT NULL,
  reserves_rowid INTEGER NOT NULL,
  -- The input amount of asset 1.
  delta_1 NUMERIC(39) NOT NULL,
  -- The input amount of asset 2.
  delta_2 NUMERIC(39) NOT NULL,
  -- The output amount of asset 1.
  lambda_1 NUMERIC(39) NOT NULL,
  -- The output amount of asset 2.
  lambda_2 NUMERIC(39) NOT NULL,
  -- The fee amount paid in asset 1.  
  fee_1 NUMERIC(39) NOT NULL,
  -- The fee amount paid in asset 2.
  fee_2 NUMERIC(39) NOT NULL,
  -- The context the execution happened in
  context_asset_start BYTEA NOT NULL,
  context_asset_end BYTEA NOT NULL
);

CREATE INDEX ON dex_ex_position_executions (height);
CREATE INDEX ON dex_ex_position_executions (position_id, height, rowid);

CREATE TABLE IF NOT EXISTS dex_ex_position_withdrawals (
  rowid SERIAL PRIMARY KEY,
  position_id BYTEA NOT NULL,
  height INTEGER NOT NULL,
  time TIMESTAMPTZ NOT NULL,
  withdrawal_tx BYTEA,
  sequence INTEGER NOT NULL,
  reserves_rowid INTEGER NOT NULL,
  -- The amount of asset 1 withdrawn.
  reserves_1 NUMERIC(39) NOT NULL,
  -- The amount of asset 2 withdrawn.
  reserves_2 NUMERIC(39) NOT NULL
);

CREATE INDEX ON dex_ex_position_withdrawals (height);
CREATE INDEX ON dex_ex_position_withdrawals (position_id, height);

-- This table tracks individual execution traces for a directed batch swap.
CREATE TABLE IF NOT EXISTS dex_ex_batch_swap_traces (
  -- Primary key
  rowid SERIAL PRIMARY KEY,

  -- The height of the block the batch swap was included in.
  height INTEGER NOT NULL,
  -- The time the batch swap was included in a block.
  time TIMESTAMPTZ NOT NULL,

  -- The amount of asset 1 consumed by the micro execution in raw denom.
  input NUMERIC(39) NOT NULL,
  -- The amount of asset 2 produced by the micro execution in raw denom.
  output NUMERIC(39) NOT NULL,

  -- The amount of asset 1 consumed by the macro execution.
  batch_input NUMERIC(39) NOT NULL,
  -- The amount of asset 2 produced by the macro execution.
  batch_output NUMERIC(39) NOT NULL,
  -- The price (output/input) as a float
  price_float DOUBLE PRECISION NOT NULL,


  -- The directed start asset of the batch swap.
  asset_start BYTEA NOT NULL,
  -- The directed end asset of the batch swap.
  asset_end BYTEA NOT NULL,

  -- Each hop in the list contains an asset id.
  asset_hops BYTEA[] NOT NULL,
  -- Each hop in the list contains an amount.
  amount_hops NUMERIC(39)[] NOT NULL,
  -- Each hop in the list contains a position ID.
  position_id_hops BYTEA[] NOT NULL
);

CREATE INDEX ON dex_ex_batch_swap_traces (time, height);
CREATE INDEX ON dex_ex_batch_swap_traces (asset_start, asset_end);
-- TODO(erwan): We can add a GIN index on the position id later.

-- A summary of block data with a bias for DEX data.
CREATE TABLE IF NOT EXISTS dex_ex_block_summary (
    -- Primary key
    rowid SERIAL PRIMARY KEY,
    -- The height of the block.
    height INTEGER NOT NULL,
    -- The timestamp for the block.
    time TIMESTAMPTZ NOT NULL,
    -- A list of batch swap summaries that occurred in this block.
    batch_swaps jsonb NOT NULL,
    -- The number of opened LPs in this block.
    num_open_lps     INTEGER NOT NULL,
    -- The number of closed LPs in this block.
    num_closed_lps   INTEGER NOT NULL,
    -- The number of withdrawn LPs in this block.
    num_withdrawn_lps INTEGER NOT NULL,
    -- The number of swaps in this block.
    num_swaps        INTEGER NOT NULL,
    -- The number of swap claims in this block.
    num_swap_claims  INTEGER NOT NULL,
    -- The number of transactions in this block.
    num_txs          INTEGER NOT NULL
);

CREATE INDEX ON dex_ex_block_summary (time, height);

CREATE TABLE IF NOT EXISTS dex_ex_transactions (
  -- The unique identifier of the transaction
  transaction_id BYTEA NOT NULL PRIMARY KEY,
  -- The raw transaction bytes
  transaction BYTEA NOT NULL,
  -- The block height at which this transaction was included
  height INTEGER NOT NULL,
  -- The timestamp when this transaction was included in a block
  time TIMESTAMPTZ NOT NULL
);

CREATE INDEX ON dex_ex_transactions (time, height);

ALTER TABLE dex_ex_position_executions
  ADD CONSTRAINT fk_position_executions
  FOREIGN KEY (position_id) REFERENCES dex_ex_position_state(position_id);

ALTER TABLE dex_ex_position_withdrawals
  ADD CONSTRAINT fk_position_withdrawals
  FOREIGN KEY (position_id) REFERENCES dex_ex_position_state(position_id);

ALTER TABLE dex_ex_position_executions
  ADD CONSTRAINT fk_position_executions_reserves
  FOREIGN KEY (reserves_rowid) REFERENCES dex_ex_position_reserves(rowid);

ALTER TABLE dex_ex_position_state
  ADD CONSTRAINT fk_position_state_reserves
  FOREIGN KEY (opening_reserves_rowid) REFERENCES dex_ex_position_reserves(rowid);

ALTER TABLE dex_ex_position_withdrawals
  ADD CONSTRAINT fk_position_withdrawals_reserves
  FOREIGN KEY (reserves_rowid) REFERENCES dex_ex_position_reserves(rowid);
