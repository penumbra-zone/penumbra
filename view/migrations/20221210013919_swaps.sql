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
