use penumbra_crypto::{merkle, note, Nullifier};

#[derive(Debug, sqlx::FromRow)]
pub struct BlobsRow {
    pub id: String,
    pub data: Vec<u8>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct BlocksRow {
    pub height: u32,
    pub nct_anchor: merkle::Root,
    pub app_hash: Vec<u8>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct NotesRow {
    pub note_commitment: note::Commitment,
    pub ephemeral_key: Vec<u8>,
    pub encrypted_note: Vec<u8>,
    pub transaction_id: Vec<u8>,
    pub height: u32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct NullifiersRow {
    pub nullifier: Nullifier,
}
