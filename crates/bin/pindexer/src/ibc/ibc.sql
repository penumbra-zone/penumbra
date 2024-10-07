CREATE TABLE IF NOT EXISTS ibc_transfer (
  id SERIAL PRIMARY KEY,
  -- The height that this transfer happened at.
  height BIGINT NOT NULL,
  -- The AssetID of whatever is being transferred.
  asset BYTEA NOT NULL,
  -- The amount being transf
  amount NUMERIC(39, 0) NOT NULL,
  -- The address on the penumbra side.
  --
  -- This may be the sender or the receiver, depending on if this inflow or outflow.
  penumbra_addr BYTEA NOT NULL,
  -- The address on the other side.
  foreign_addr TEXT NOT NULL,
  -- What kind of transfer this is.
  kind TEXT NOT NULL CHECK (kind IN ('inbound', 'outbound', 'refund_timeout', 'refund_error', 'refund_other'))
);
