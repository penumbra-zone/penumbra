CREATE TABLE IF NOT EXISTS auction_dutch_description (
  -- The auction identifier.
  id BYTEA PRIMARY KEY,
  -- The input asset.
  in_asset BYTEA NOT NULL,
  -- The input amount.
  in_amount NUMERIC(39, 0) NOT NULL,
  -- The desired output asset.
  out_asset BYTEA NOT NULL,
  -- The maximum amount of output to get.
  out_max NUMERIC(39, 0) NOT NULL,
  -- The minimum amount of output to get.
  out_min NUMERIC(39, 0) NOT NULL,
  -- The height the auction starts at.
  start_height BIGINT NOT NULL,
  -- The height the auction ends at.
  end_height BIGINT NOT NULL,
  -- The number of steps to take.
  step_count BIGINT NOT NULL,
  nonce BYTEA NOT NULL
);

-- To easily lookup auctions by ascending input amount.
CREATE INDEX ON auction_dutch_description(in_asset, in_amount);

CREATE TABLE IF NOT EXISTS auction_dutch_update (
  -- Serves only as a primary key.
  id SERIAL PRIMARY KEY,
  -- Points to the description of the auction.
  auction_id BYTE NOT NULL REFERENCES auction_dutch_description (id),
  -- The height of this update.
  height BIGINT NOT NULL,
  -- 0 -> opened, 1 -> closed, >= 2 -> withdrawn.
  state INT NOT NULL,
  -- 0 -> no reason, 1 -> expired, 2 -> filled, 3 -> closed
  ended_reason INT,
  -- If present, the current position being managed by the auction.
  position_id BYTEA,
  -- The next height at which a change to the positions will happen.
  next_trigger INT,
  -- The current reserves of the input asset.
  in_reserves NUMERIC(39, 0) NOT NULL,
  -- The current reserves of the output asset.
  out_reserves NUMERIC(39, 0) NOT NULL
);
-- To be able to easily find updates for a given auction.
CREATE INDEX ON auction_dutch_update(auction_id, height);

