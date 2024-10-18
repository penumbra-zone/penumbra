-- A table containing updates to the total supply, and market cap.
CREATE TABLE IF NOT EXISTS insights_supply (
    -- The height where the supply was updated.
    height BIGINT PRIMARY KEY,
    -- The total supply of the staking token at this height.
    total BIGINT NOT NULL,   
    staked BIGINT NOT NULL,
    -- Price, if it can be found for whatever numeraire we choose at runtime.
    price FLOAT8,
    -- The numeraire for the price we've chosen.
    price_numeraire_asset_id BYTEA,
    -- The market cap, i.e. price * total amount.
    market_cap FLOAT8 GENERATED ALWAYS AS (total::FLOAT8 * price) STORED
);

-- A working table to save the state around validators we need.
--
-- This is necessary because rate data changes increase the total supply,
-- but don't directly tell us how much the total supply increased.
CREATE TABLE IF NOT EXISTS _insights_validators (
    -- The validator this row concerns.
    validator_id TEXT NOT NULL,
    -- The height for the supply update.
    height BIGINT NOT NULL,
    -- The total amount staked with them, in terms of the native token.
    um BIGINT NOT NULL,   
    -- How much native um we get per unit of the delegation token.
    rate_bps2 BIGINT NOT NULL,
    PRIMARY KEY (validator_id, height)
);

-- Our internal representation of the shielded pool table.
CREATE TABLE IF NOT EXISTS insights_shielded_pool (
    -- The asset this concerns.
    asset_id BYTEA NOT NULL,  
    height BIGINT NOT NULL,
    -- The total value shielded, in terms of that asset.
    total_value TEXT NOT NULL,
    -- The current value shielded, in terms of that asset.
    current_value TEXT NOT NULL,
    -- The number of unique depositors.
    unique_depositors INT NOT NULL,
    PRIMARY KEY (asset_id, height)
);

-- Unique depositors into the shielded pool
CREATE TABLE IF NOT EXISTS _insights_shielded_pool_depositors (
    asset_id BYTEA NOT NULL,
    address TEXT NOT NULL,
    PRIMARY KEY (asset_id, address)
);

CREATE OR REPLACE VIEW insights_shielded_pool_latest AS
    SELECT DISTINCT ON (asset_id) * FROM insights_shielded_pool ORDER BY asset_id, height DESC;
