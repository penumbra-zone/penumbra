CREATE TABLE IF NOT EXISTS notes (
    note_commitment bytea PRIMARY KEY,
    ephemeral_key bytea NOT NULL,
    encrypted_note bytea NOT NULL,
    transaction_id bytea NOT NULL,
    height bigint REFERENCES blocks (height)
)