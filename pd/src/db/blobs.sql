CREATE TABLE IF NOT EXISTS blobs (
    id varchar(64) PRIMARY KEY,
    data bytea NOT NULL
)