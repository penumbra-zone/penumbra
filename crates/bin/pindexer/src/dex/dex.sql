-- This component is responsible for processing events related to the DEX.

-- # Design Choices 
--
-- ## Asset IDs
--
-- We represent them as raw bytes---i.e. BYTEA---, rather than using a 1:1 table.
-- This is probably more efficient, and makes our lives much easier by the fact
-- that given an `penumbra_asset::asset::Id`, we always know exactly how to filter
-- tables, rather than needing to do a join with another table.

-- Keeps track of changes to the dex's value circuit breaker.
CREATE TABLE IF NOT EXISTS dex_value_circuit_breaker_change (
  -- The asset being moved into or out of the dex.
  asset_id BYTEA NOT NULL,
  -- The flow, either positive, or negative, into the dex via this particular asset.
  --
  -- Because we're dealing with arbitrary assets, we need to use something which can store u128
  flow NUMERIC(39, 0) NOT NULL
);
