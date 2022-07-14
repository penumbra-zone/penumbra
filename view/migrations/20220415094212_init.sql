-- Application state, stored in single-row tables
CREATE TABLE chain_params (bytes BLOB NOT NULL);
CREATE TABLE full_viewing_key (bytes BLOB NOT NULL);
CREATE TABLE sync_height (height BIGINT NOT NULL);
CREATE TABLE note_commitment_tree (bytes BLOB NOT NULL);

-- Minimal data required for balance tracking
-- Meant to represent notes which have been accepted into the note set
CREATE TABLE notes (
    note_commitment         BLOB PRIMARY KEY NOT NULL,
    height_spent            BIGINT, --null if unspent, otherwise spent at height_spent 
    height_created          BIGINT NOT NULL,
    -- note contents themselves:
    diversifier             BLOB NOT NULL,
    amount                  BIGINT NOT NULL,
    asset_id                BLOB NOT NULL,
    transmission_key        BLOB NOT NULL,
    blinding_factor         BLOB NOT NULL,
    -- precomputed decryption of the diversifier
    address_index       BLOB NOT NULL,
    -- the nullifier for this note, used to detect when it is spent
    nullifier               BLOB NOT NULL,
    -- the position of the note in the note commitment tree
    position                BIGINT NOT NULL
);

-- general purpose note queries
CREATE INDEX notes_idx ON notes (
    height_spent,       -- null if unspent, so spent/unspent is first
    address_index,  -- then filter by account
    asset_id,           -- then by asset
    amount,             -- then by amount
    height_created      -- we don't really care about this, except informationally
);

-- used to detect spends
CREATE INDEX nullifier_idx on notes ( nullifier );

-- used for storing a cache of known assets
CREATE TABLE assets (
    asset_id BLOB PRIMARY KEY NOT NULL,
    denom    TEXT NOT NULL
);

CREATE TABLE quarantined_notes (
    note_commitment         BLOB PRIMARY KEY NOT NULL,
    height_created          BIGINT NOT NULL,
    -- note contents themselves:
    diversifier             BLOB NOT NULL,
    amount                  BIGINT NOT NULL,
    asset_id                BLOB NOT NULL,
    transmission_key        BLOB NOT NULL,
    blinding_factor         BLOB NOT NULL,
    -- precomputed decryption of the diversifier
    address_index       BLOB NOT NULL,
    -- the quarantine status of the note
    unbonding_epoch         BIGINT NOT NULL,
    identity_key            BLOB NOT NULL
);

CREATE INDEX quarantined_notes_idx ON quarantined_notes (
    identity_key,       -- first by identity key
    unbonding_epoch,    -- then by unbonding epoch
    address_index,  -- then filter by account
    amount,             -- then by amount
    height_created      -- we don't really care about this, except informationally
);

-- Nullifiers and identity keys added here in the event of a provisional spend (height_spent updated in notes table, then provisionality of that spend is recorded here)
CREATE TABLE quarantined_nullifiers (
    nullifier               BLOB PRIMARY KEY NOT NULL,
    identity_key            BLOB NOT NULL
);

CREATE INDEX identity_key_idx ON quarantined_nullifiers (
    identity_key
);

CREATE TABLE tx_by_nullifier (
    nullifier               BLOB PRIMARY KEY NOT NULL,
    tx_hash                 BLOB NOT NULL
);

CREATE TABLE tx_by_note_commitment (
    note_commitment         BLOB PRIMARY KEY NOT NULL,
    tx_hash                 BLOB NOT NULL
);

CREATE TABLE tx (
    tx_hash                 BLOB PRIMARY KEY NOT NULL,
    tx_body                 BLOB NOT NULL
);