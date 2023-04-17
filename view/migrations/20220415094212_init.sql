-- Application state, stored in single-row tables
CREATE TABLE chain_params (bytes BLOB NOT NULL);
CREATE TABLE fmd_parameters (bytes BLOB NOT NULL);
CREATE TABLE full_viewing_key (bytes BLOB NOT NULL);
CREATE TABLE sync_height (height BIGINT NOT NULL);

-- used for storing a cache of known assets
CREATE TABLE assets (
    asset_id                BLOB PRIMARY KEY NOT NULL,
    denom                   TEXT NOT NULL
);