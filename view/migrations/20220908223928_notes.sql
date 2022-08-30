CREATE TABLE notes (
    note_commitment         BLOB PRIMARY KEY NOT NULL,
    address                 BLOB NOT NULL,
    amount                  BIGINT NOT NULL,
    asset_id                BLOB NOT NULL,
    blinding_factor         BLOB NOT NULL,
    height_created          BIGINT NOT NULL,
    -- precomputed decryption of the diversifier
    address_index           BLOB NOT NULL,
    -- the source of the note (a tx hash or structured data jammed into one)
    source                  BLOB NOT NULL
);

-- general purpose note queries
CREATE INDEX notes_idx ON notes (
    address,            
    asset_id,           
    amount,
    height_created,
    address_index,
    source
);

-- Minimal data required for balance tracking
-- Meant to represent notes which have been accepted into the note set
CREATE TABLE spendable_notes (
    note_commitment         BLOB PRIMARY KEY NOT NULL,
    --null if unspent, otherwise spent at height_spent 
    height_spent            BIGINT, 
    -- the nullifier for this note, used to detect when it is spent
    nullifier               BLOB NOT NULL,
    -- the position of the note in the note commitment tree
    position                BIGINT NOT NULL
);

-- general purpose note queries
CREATE INDEX spendable_notes_idx ON spendable_notes (
    height_spent,       -- null if unspent, so spent/unspent is first
    nullifier
);

CREATE TABLE quarantined_notes (
    note_commitment         BLOB PRIMARY KEY NOT NULL,
    -- the quarantine status of the note
    unbonding_epoch         BIGINT NOT NULL,
    identity_key            BLOB NOT NULL
);

CREATE INDEX quarantined_notes_idx ON quarantined_notes (
    identity_key,       -- first by identity key
    unbonding_epoch     -- then by unbonding epoch
);

-- Nullifiers and identity keys added here in the event of a provisional spend (height_spent updated in notes table, then provisionality of that spend is recorded here)
CREATE TABLE quarantined_nullifiers (
    nullifier               BLOB PRIMARY KEY NOT NULL,
    identity_key            BLOB NOT NULL
);

CREATE INDEX identity_key_idx ON quarantined_nullifiers (
    identity_key
);