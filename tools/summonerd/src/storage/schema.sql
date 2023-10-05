-- The hash of this schema file
CREATE TABLE schema_hash (schema_hash TEXT NOT NULL);

-- used for storing phase 2 contribution information
CREATE TABLE phase2_contributions (
    number                     BIGINT PRIMARY KEY NOT NULL,
    address                    BLOB NOT NULL
);
