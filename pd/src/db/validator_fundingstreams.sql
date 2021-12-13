CREATE TABLE IF NOT EXISTS validator_fundingstreams (
    tm_pubkey bytea NOT NULL,
    address varchar NOT NULL,
    rate_bps bigint NOT NULL
)