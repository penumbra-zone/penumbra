CREATE TABLE IF NOT EXISTS nullifiers (
    nullifier bytea PRIMARY KEY,
    height bigint NOT NULL REFERENCES blocks (height)
)