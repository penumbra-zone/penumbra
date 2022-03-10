-- Miscellaneous key-value storage for the application
CREATE TABLE IF NOT EXISTS blobs (
    id varchar(64) PRIMARY KEY,
    data bytea NOT NULL
);

-- Backing storage for the public Jellyfish Merkle Tree
CREATE TABLE IF NOT EXISTS jmt (
    key bytea PRIMARY KEY,
    value bytea NOT NULL
);

-- All known assets
CREATE TABLE IF NOT EXISTS assets (
    asset_id bytea PRIMARY KEY NOT NULL,
    denom varchar NOT NULL,
    total_supply bigint NOT NULL,
    -- total supply can't be negative
    CONSTRAINT positive_total_supply CHECK (total_supply >= 0)
);

-- Blocks, indexed by height
CREATE TABLE IF NOT EXISTS blocks (
    height bigint PRIMARY KEY,
    nct_anchor bytea NOT NULL,
    app_hash bytea NOT NULL,
    -- height can't be negative
    CONSTRAINT positive_height CHECK (height >= 0)
);

-- Nullifiers, indexed by height
CREATE TABLE IF NOT EXISTS nullifiers (
    nullifier bytea PRIMARY KEY,
    height bigint NOT NULL REFERENCES blocks (height)
);
CREATE INDEX ON nullifiers (height);

-- Notes that appear in each block, indexed by height and position
CREATE TABLE IF NOT EXISTS notes (
    note_commitment bytea PRIMARY KEY,
    ephemeral_key bytea NOT NULL,
    encrypted_note bytea NOT NULL,
    transaction_id bytea NOT NULL,
    position bigint NOT NULL,
    height bigint NOT NULL REFERENCES blocks (height),
    -- position can't be negative
    CONSTRAINT positive_position CHECK (position >= 0)
);
CREATE INDEX ON notes (position);
CREATE INDEX ON notes (height);

-- All validators who have ever been declared
CREATE TABLE IF NOT EXISTS validators (
    identity_key bytea NOT NULL PRIMARY KEY,
    consensus_key bytea NOT NULL,
    sequence_number bigint NOT NULL,
    name varchar NOT NULL,
    website varchar NOT NULL,
    description varchar NOT NULL,
    voting_power bigint NOT NULL,
    validator_state varchar NOT NULL,
    unbonding_epoch bigint,
    -- sequence_number can't be negative
    CONSTRAINT positive_sequence_number CHECK (sequence_number >= 0),
    -- voting power can't be negative
    CONSTRAINT positive_voting_power CHECK (voting_power >= 0),
    -- validator state can only be one of the valid strings
    CONSTRAINT valid_state_name
        CHECK (validator_state IN ('INACTIVE', 'ACTIVE', 'UNBONDING', 'SLASHED')),
    -- the unbonding epoch is not null precisely when the validator is unbonding
    CONSTRAINT unbonding_epoch_exactly_when_unbonding
        CHECK ((validator_state =  'UNBONDING' AND unbonding_epoch IS NOT NULL) OR
               (validator_state != 'UNBONDING' AND unbonding_epoch IS NULL))
);

-- The funding streams for all validators who have ever been declared
CREATE TABLE IF NOT EXISTS validator_fundingstreams (
    identity_key bytea NOT NULL REFERENCES validators (identity_key),
    address varchar NOT NULL,
    rate_bps bigint NOT NULL,
    -- rate_bps must range between [0, 10000] inclusive for it to represent a percentage rate
    CONSTRAINT valid_bps CHECK (rate_bps >= 0 AND rate_bps <= 10000)
);

-- The base reward rate for each epoch
CREATE TABLE IF NOT EXISTS base_rates (
    epoch bigint PRIMARY KEY,
    base_reward_rate bigint NOT NULL,
    base_exchange_rate bigint NOT NULL,
    -- epoch can't be negative
    CONSTRAINT positive_epoch CHECK (epoch >= 0)
);

-- The validator rates for each epoch (some may be skipped if the validator was not active)
CREATE TABLE IF NOT EXISTS validator_rates (
    identity_key bytea NOT NULL REFERENCES validators (identity_key),
    epoch bigint NOT NULL,
    validator_reward_rate bigint NOT NULL,
    validator_exchange_rate bigint NOT NULL,
    PRIMARY KEY(epoch, identity_key),
    -- epoch can't be negative'
    CONSTRAINT positive_epoch CHECK (epoch >= 0)
);

-- Changes to delegations that occurred in each block
CREATE TABLE IF NOT EXISTS delegation_changes (
    validator_identity_key bytea NOT NULL REFERENCES validators (identity_key),
    epoch bigint NOT NULL,
    delegation_change bigint NOT NULL,
    -- epoch can't be negative
    CONSTRAINT positive_epoch CHECK (epoch >= 0)
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
    unbonding_height bigint NOT NULL, -- height at which to make the note available
    validator_identity_key bytea NOT NULL REFERENCES validators (identity_key),
    -- unbonding_height can't be negative
    CONSTRAINT positive_unbonding_height CHECK (unbonding_height >= 0)
);
CREATE INDEX ON quarantined_notes (unbonding_height);
CREATE INDEX ON quarantined_notes (validator_identity_key);

-- Set of quarantined nullifiers: once the epoch is large enough, all old-enough quarantined
-- nullifiers should be dropped from this table (they are already in the nullifiers table),
-- making the spend which revealed them irreversible
CREATE TABLE IF NOT EXISTS quarantined_nullifiers (
    nullifier bytea PRIMARY KEY REFERENCES nullifiers (nullifier),
    unbonding_height bigint NOT NULL, -- height at which to make the spend permanent
    validator_identity_key bytea NOT NULL REFERENCES validators (identity_key),
    -- unbonding_height can't be negative
    CONSTRAINT positive_unbonding_height CHECK (unbonding_height >= 0)
);
CREATE INDEX ON quarantined_nullifiers (unbonding_height);
CREATE INDEX ON quarantined_nullifiers (validator_identity_key);
