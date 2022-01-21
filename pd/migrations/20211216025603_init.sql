CREATE TABLE IF NOT EXISTS blobs (
    id varchar(64) PRIMARY KEY,
    data bytea NOT NULL
);

CREATE TABLE IF NOT EXISTS assets (
    asset_id bytea PRIMARY KEY NOT NULL,
    denom varchar NOT NULL,
    total_supply bigint NOT NULL
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
    identity_key bytea NOT NULL PRIMARY KEY,
    consensus_key bytea NOT NULL,
    sequence_number bigint NOT NULL,
    validator_data bytea NOT NULL,
    voting_power bigint NOT NULL,
    -- TODO: validator_state should be limited to valid state strings
    validator_state varchar NOT NULL,
    -- TODO: unbonding epoch can only be set when validator_state is unbonding
    unbonding_epoch bigint
);

CREATE TABLE IF NOT EXISTS validator_fundingstreams (
    identity_key bytea NOT NULL REFERENCES validators (identity_key),
    address varchar NOT NULL,
    rate_bps bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS base_rates (
    epoch bigint PRIMARY KEY,
    base_reward_rate bigint NOT NULL,
    base_exchange_rate bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS validator_rates (
    identity_key bytea NOT NULL REFERENCES validators (identity_key),
    epoch bigint NOT NULL,
    validator_reward_rate bigint NOT NULL,
    validator_exchange_rate bigint NOT NULL,
    PRIMARY KEY(epoch, identity_key)
);

CREATE TABLE IF NOT EXISTS delegation_changes (
    validator_identity_key bytea NOT NULL REFERENCES validators (identity_key),
    epoch bigint NOT NULL,
    delegation_change bigint NOT NULL
);
CREATE INDEX ON delegation_changes (epoch);
CREATE INDEX ON delegation_changes (validator_identity_key);

-- Set of quarantined notes: once the epoch is large enough, all old-enough quarantined notes should
-- be inserted into the NCT and the notes table (and removed from this table)
CREATE TABLE IF NOT EXISTS quarantined_notes (
    note_commitment bytea PRIMARY KEY,
    ephemeral_key bytea NOT NULL,
    encrypted_note bytea NOT NULL,
    transaction_id bytea NOT NULL,
    height bigint NOT NULL REFERENCES blocks (height),
    validator_identity_key bytea NOT NULL REFERENCES validators (identity_key)
);
CREATE INDEX ON quarantined_notes (height);
CREATE INDEX ON quarantined_notes (validator_identity_key);

-- Set of quarantined nullifiers: once the epoch is large enough, all old-enough quarantined
-- nullifiers should be dropped from this table (they are already in the nullifiers table),
-- making the spend which revealed them irreversible
CREATE TABLE IF NOT EXISTS quarantined_nullifiers (
    nullifier bytea PRIMARY KEY REFERENCES nullifiers (nullifier),
    height bigint NOT NULL REFERENCES blocks (height),
    validator_identity_key bytea NOT NULL REFERENCES validators (identity_key)
);
CREATE INDEX ON quarantined_nullifiers (height);
CREATE INDEX ON quarantined_nullifiers (validator_identity_key);