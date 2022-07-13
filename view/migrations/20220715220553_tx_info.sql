-- Add migration script here

CREATE TABLE tx_by_nullifier (
    nullifier               BLOB PRIMARY KEY NOT NULL,
    tx_hash                 BLOB NOT NULL
);

CREATE TABLE tx (
    tx_hash                 BLOB PRIMARY KEY NOT NULL,
    tx_bytes                BLOB NOT NULL
);
