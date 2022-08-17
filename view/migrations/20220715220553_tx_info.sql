-- Add migration script here

CREATE TABLE tx_by_nullifier (
    nullifier               BLOB PRIMARY KEY NOT NULL,
    tx_hash                 BLOB NOT NULL
);

CREATE TABLE tx (
    tx_hash                 BLOB PRIMARY KEY NOT NULL,
    binding_sig             BLOB NOT NULL,
    anchor                  BLOB NOT NULL,
    expiry_height           INTEGER NOT NULL,
    chain_id                TEXT NOT NULL,
    fee                     INTEGER NOT NULL
);

CREATE TABLE tx_actions (
    tx_hash                 BLOB PRIMARY KEY NOT NULL,
    action_type             TEXT NOT NULL
);

CREATE TABLE tx_clues (
    tx_hash                 BLOB PRIMARY KEY NOT NULL,
    clue                    BLOB NOT NULL
);