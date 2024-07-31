-- This component is responsible for processing events related to the DEX.

-- # Design Choices 
--
-- ## Asset IDs
--
-- We represent them as raw bytes---i.e. BYTEA---, rather than using a 1:1 table.
-- This is probably more efficient, and makes our lives much easier by the fact
-- that given an `penumbra_asset::asset::Id`, we always know exactly how to filter
-- tables, rather than needing to do a join with another table.


DROP DOMAIN IF EXISTS Amount;
CREATE DOMAIN Amount AS NUMERIC(39, 0) NOT NULL;

DROP TYPE IF EXISTS Value CASCADE;
CREATE TYPE Value AS
(
    amount Amount,
    asset  BYTEA
);

-- Keeps track of changes to the dex's value circuit breaker.
CREATE TABLE IF NOT EXISTS dex_value_circuit_breaker_change
(
    -- The asset being moved into or out of the dex.
    asset_id BYTEA NOT NULL,
    -- The flow, either positive, or negative, into the dex via this particular asset.
    --
    -- Because we're dealing with arbitrary assets, we need to use something which can store u128
    flow     Amount
);

-- One step of an execution trace.
CREATE TABLE IF NOT EXISTS dex_trace_step (
  id SERIAL PRIMARY KEY,
  value Value
);

-- A single trace, showing what a small amount of an input asset was exchanged for.
CREATE TABLE IF NOT EXISTS dex_trace (
  id SERIAL PRIMARY KEY,
  step_start INTEGER REFERENCES dex_trace_step(id),
  step_end INTEGER REFERENCES dex_trace_step(id)
);

--- Represents instances where arb executions happened.
CREATE TABLE IF NOT EXISTS dex_arb (
  height BIGINT PRIMARY KEY,
  input Value,
  output Value,
  trace_start INTEGER REFERENCES dex_trace(id),
  trace_end INTEGER REFERENCES dex_trace(id)
);

DROP DOMAIN IF EXISTS Bps;
CREATE DOMAIN Bps AS INTEGER CHECK(VALUE BETWEEN 0 and 10000);

-- Holds the current state of a given liquidity position
CREATE TABLE IF NOT EXISTS dex_lp (
  id BYTEA PRIMARY KEY,
  -- The enum for the current state of the position
  state TEXT NOT NULL,
  -- The first asset of the position
  asset1 BYTEA NOT NULL,
  -- The second asset of the position
  asset2 BYTEA NOT NULL,
  p Amount,
  q Amount,
  -- The fee, in basis points
  fee_bps Bps,
  -- How much of asset2 you get when swapping asset1.
  price12 NUMERIC GENERATED ALWAYS AS (((1 - fee_bps::NUMERIC / 10000) * (p / q))) STORED,
  -- How much of asset1 you get when swapping asset2.
  price21 NUMERIC GENERATED ALWAYS AS (((1 - fee_bps::NUMERIC / 10000) * (q / p))) STORED,
  -- Whether the position will be closed when all reserves are depleted
  close_on_fill BOOLEAN NOT NULL,
  -- The amount of reserves of asset 1.
  reserves1 Amount,
  -- The amount of reserves of asset 2.
  reserves2 Amount
);
-- So that we can easily query positions by ascending price
CREATE INDEX ON dex_lp(asset1, price12);
CREATE INDEX ON dex_lp(asset2, price21);

-- Holds update events to liquidity position
CREATE TABLE IF NOT EXISTS dex_lp_update (
  id SERIAL PRIMARY KEY,
  -- The block height where the update occurred.
  height BIGINT NOT NULL,
  -- The identifier for the position
  position_id BYTEA NOT NULL,
  -- The new state of the position
  state TEXT NOT NULL,
  -- The new reserves of asset1 (potentially null)
  reserves1 NUMERIC(39, 0),
  -- The new reserves of asset2 (potentially null)
  reserves2 NUMERIC(39, 0),
  -- If present, a reference to the execution table, for execution events
  execution_id INTEGER
);
CREATE INDEX ON dex_lp_update(position_id, height DESC, id DESC);
CREATE INDEX ON dex_lp_update(execution_id);

-- Holds data related to execution events
CREATE TABLE IF NOT EXISTS dex_lp_execution (
  id SERIAL PRIMARY KEY,
  -- The amount of asset1 that was pushed into (or pulled out of, if negative) this position.
  inflow1 Amount,
  -- As above, with asset2.
  inflow2 Amount,
  -- The start asset for this execution.
  context_start BYTEA NOT NULL,
  -- The end asset for this execution.
  context_end BYTEA NOT NULL
);

--- Represents instances where swap executions happened.
CREATE TABLE IF NOT EXISTS dex_swap
(
    height      BIGINT PRIMARY KEY,
    trace_start INTEGER REFERENCES dex_trace (id),
    trace_end   INTEGER REFERENCES dex_trace (id),
    pair_asset_id_1  BYTEA NOT NULL,
    pair_asset_id_2  BYTEA NOT NULL,
    unfilled_1 Amount NOT NULL,
    unfilled_2 Amount NOT NULL,
    delta_1 Amount NOT NULL,
    delta_2 Amount NOT NULL,
    lambda_1 Amount NOT NULL,
    lambda_2 Amount NOT NULL
);