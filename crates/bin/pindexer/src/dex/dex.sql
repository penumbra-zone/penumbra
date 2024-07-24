-- This component is responsible for processing events related to the DEX.

-- # Design Choices 
--
-- ## Asset IDs
--
-- We represent them as raw bytes---i.e. BYTEA---, rather than using a 1:1 table.
-- This is probably more efficient, and makes our lives much easier by the fact
-- that given an `penumbra_asset::asset::Id`, we always know exactly how to filter
-- tables, rather than needing to do a join with another table.

CREATE DOMAIN IF NOT EXISTS Amount AS NUMERIC (39, 0) NOT NULL;

CREATE TYPE Value AS
(
    amount Amount,
    asset  BYTEA NOT NULL
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
CREATE TABLE IF NOT EXISTS trace_step
(
    id    SERIAL PRIMARY KEY,
    value Value,
);

-- A single trace, showing what a small amount of an input asset was exchanged for.
CREATE TABLE IF NOT EXISTS trace
(
    id         SERIAL PRIMARY KEY,
    step_start INTEGER REFERENCES trace_step (id),
    step_end   INTEGER REFERENCES trace_step (id),
);

--- Represents instances where arb executions happened.
CREATE TABLE IF NOT EXISTS arb
(
    height      BIGINT PRIMARY KEY,
    input       Value,
    output      Value,
    trace_start INTEGER REFERENCES arb_traces (id),
    trace_end   INTEGER REFERENCES arb_traces (id),
);

--- Represents LP updates
CREATE TABLE IF NOT EXISTS lp_updates
(
    id           SERIAL PRIMARY KEY,
    height       INT8    NOT NULL,
    type         integer NOT NULL,
    position_id  BYTEA   NOT NULL,
    trading_pair BYTEA
);
