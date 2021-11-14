pub mod schema;

use anyhow::Result;
use sqlx::{query, Pool, Postgres};
use tracing::instrument;

/// Create database tables if they do not already exist.
#[instrument]
pub async fn init_tables(db: &Pool<Postgres>) -> Result<()> {
    query(
        r#"
CREATE TABLE IF NOT EXISTS blobs (
    id varchar(64) PRIMARY KEY,
    data bytea NOT NULL
)
"#,
    )
    .execute(db)
    .await?;

    query(
        r#"
CREATE TABLE IF NOT EXISTS blocks (
    height bigint PRIMARY KEY, 
    nct_anchor bytea NOT NULL,
    app_hash bytea NOT NULL
)
"#,
    )
    .execute(db)
    .await?;

    query(
        r#"
CREATE TABLE IF NOT EXISTS notes (
    note_commitment bytea PRIMARY KEY,
    ephemeral_key bytea NOT NULL,
    encrypted_note bytea NOT NULL,
    transaction_id bytea NOT NULL,
    height bigint REFERENCES blocks (height)
)
"#,
    )
    .execute(db)
    .await?;

    query(
        r#"
CREATE TABLE IF NOT EXISTS nullifiers (
    nullifier bytea PRIMARY KEY
)
"#,
    )
    .execute(db)
    .await?;

    Ok(())
}
