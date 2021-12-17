-- Add migration script here
CREATE TABLE IF NOT EXISTS blobs (
    id varchar(64) PRIMARY KEY,
    data bytea NOT NULL
);

CREATE TABLE IF NOT EXISTS assets (
    asset_id bytea PRIMARY KEY NOT NULL,
    denom varchar NOT NULL
);

CREATE TABLE IF NOT EXISTS blocks (
    height bigint PRIMARY KEY,
    nct_anchor bytea NOT NULL,
    app_hash bytea NOT NULL
);

CREATE TABLE IF NOT EXISTS nullifiers (
    nullifier bytea PRIMARY KEY,
    height bigint NOT NULL REFERENCES blocks (height)
);
CREATE INDEX ON nullifiers (height);

CREATE TABLE IF NOT EXISTS notes (
    note_commitment bytea PRIMARY KEY,
    ephemeral_key bytea NOT NULL,
    encrypted_note bytea NOT NULL,
    transaction_id bytea NOT NULL,
    position bigint NOT NULL,
    height bigint NOT NULL REFERENCES blocks (height)
);
CREATE INDEX ON notes (position);
CREATE INDEX ON notes (height);

CREATE TABLE IF NOT EXISTS validators (
    tm_pubkey bytea NOT NULL PRIMARY KEY,
    voting_power bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS validator_fundingstreams (
    tm_pubkey bytea NOT NULL,
    address varchar NOT NULL,
    rate_bps bigint NOT NULL
);
