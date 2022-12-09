-- This table just records the mapping from note commitments to note plaintexts.
-- This is also used as a way to give advice about out-of-band notes during scanning,
-- by allowing the user to add notes to the database before they are scanned.
CREATE TABLE notes (
    note_commitment         BLOB PRIMARY KEY NOT NULL,
    address                 BLOB NOT NULL,
    amount                  BIGINT NOT NULL,
    asset_id                BLOB NOT NULL,
    blinding_factor         BLOB NOT NULL
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
    -- the position of the note in the note commitment tree
    position                BIGINT NOT NULL,
    -- the height at which the note was created
    height_created          BIGINT NOT NULL,
    -- precomputed decryption of the diversifier
    address_index           BLOB NOT NULL,
    -- the source of the note (a tx hash or structured data jammed into one)
    source                  BLOB NOT NULL,
    --null if unspent, otherwise spent at height_spent 
    height_spent            BIGINT 
);

-- general purpose note queries
CREATE INDEX spendable_notes_idx ON spendable_notes (
    address_index,
    source,
    height_created,
    nullifier,
    height_spent       -- null if unspent, so spent/unspent is first
);