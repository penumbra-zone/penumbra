-- The hash of this schema file
CREATE TABLE schema_hash (schema_hash TEXT NOT NULL);

-- The client version that created this database
CREATE TABLE client_version (client_version TEXT NOT NULL);

-- Application state, stored in single-row tables
CREATE TABLE stake_params (bytes BLOB NOT NULL);
CREATE TABLE ibc_params (bytes BLOB NOT NULL);
CREATE TABLE governance_params (bytes BLOB NOT NULL);
CREATE TABLE chain_params (bytes BLOB NOT NULL);
CREATE TABLE community_pool_params (bytes BLOB NOT NULL);
CREATE TABLE fee_params (bytes BLOB NOT NULL);
CREATE TABLE distributions_params (bytes BLOB NOT NULL);
CREATE TABLE fmd_parameters (bytes BLOB NOT NULL);
CREATE TABLE full_viewing_key (bytes BLOB NOT NULL);
CREATE TABLE sync_height (height BIGINT NOT NULL);
CREATE TABLE gas_prices (bytes BLOB NOT NULL);

-- used for storing a cache of known assets
CREATE TABLE assets (
    asset_id                BLOB PRIMARY KEY NOT NULL,
    denom                   TEXT NOT NULL
);

-- the shape information about the sct
CREATE TABLE sct_position ( position BIGINT );
INSERT INTO sct_position VALUES ( 0 ); -- starting position is 0

CREATE TABLE sct_forgotten ( forgotten BIGINT NOT NULL );
INSERT INTO sct_forgotten VALUES ( 0 ); -- starting forgotten version is 0

-- the hashes for nodes in the sct
CREATE TABLE sct_hashes (
    position BIGINT NOT NULL,
    height   TINYINT NOT NULL,
    hash     BLOB NOT NULL
);

-- these indices may help with 2-dimensional range deletion
CREATE INDEX hash_position_idx ON sct_hashes ( position );
--CREATE INDEX hash_height_idx ON sct_hashes ( height );

-- all the commitments stored in the sct
CREATE TABLE sct_commitments (
    position BIGINT NOT NULL,
    commitment BLOB NOT NULL
);

-- look up transaction hashes by nullifier
CREATE TABLE tx_by_nullifier (
    nullifier               BLOB PRIMARY KEY NOT NULL,
    tx_hash                 BLOB NOT NULL
);

-- list of all known relevant transactions
CREATE TABLE tx (
    tx_hash                 BLOB PRIMARY KEY NOT NULL,
    tx_bytes                BLOB NOT NULL,
    block_height            BIGINT NOT NULL,
    return_address          BLOB
);

-- This table just records the mapping from note commitments to note plaintexts.
-- This is also used as a way to give advice about out-of-band notes during scanning,
-- by allowing the user to add notes to the database before they are scanned.
CREATE TABLE notes (
    note_commitment         BLOB PRIMARY KEY NOT NULL,
    address                 BLOB NOT NULL,
    amount                  BLOB NOT NULL,
    asset_id                BLOB NOT NULL,
    rseed                   BLOB NOT NULL
);

-- general purpose note queries
CREATE INDEX notes_idx ON notes (
    address,
    asset_id,
    amount
);

-- Minimal data required for balance tracking
-- Meant to represent notes which have been accepted into the note set
CREATE TABLE spendable_notes (
    note_commitment         BLOB PRIMARY KEY NOT NULL,
    -- the nullifier for this note, used to detect when it is spent
    nullifier               BLOB NOT NULL,
    -- the position of the note in the state commitment tree
    position                BIGINT NOT NULL,
    -- the height at which the note was created
    height_created          BIGINT NOT NULL,
    -- precomputed decryption of the diversifier
    address_index           BLOB NOT NULL,
    -- the source of the note (a tx hash or structured data jammed into one)
    source                  BLOB NOT NULL,
    -- null if unspent, otherwise spent at height_spent
    height_spent            BIGINT
);

CREATE INDEX spendable_notes_by_nullifier_idx ON spendable_notes (
    nullifier
);

CREATE INDEX spendable_notes_by_source_idx ON spendable_notes (
    source
);

-- general purpose note queries
CREATE INDEX spendable_notes_idx ON spendable_notes (
    address_index,
    height_created,
    height_spent       -- null if unspent, so spent/unspent is first
);

-- This table records the mapping from swap commitments to swap plaintexts.
-- For now we just store the swap plaintexts as a blob.
CREATE TABLE swaps (
    swap_commitment         BLOB PRIMARY KEY NOT NULL,
    swap                    BLOB NOT NULL,
    position                BIGINT NOT NULL,
    nullifier               BLOB NOT NULL,
    output_data             BLOB NOT NULL,
    height_claimed          BIGINT,
    source                  BLOB NOT NULL
);

CREATE INDEX swaps_nullifier_idx ON swaps (nullifier);

CREATE TABLE positions (
     position_id            BLOB PRIMARY KEY NOT NULL,
     position_state         TEXT NOT NULL,
     trading_pair           TEXT NOT NULL
);
