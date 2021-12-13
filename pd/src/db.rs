pub mod schema;

use anyhow::{Context, Result};
use sqlx::{query_file, Pool, Postgres};
use tracing::instrument;

/// Create database tables if they do not already exist.
#[instrument]
pub async fn init_tables(db: &Pool<Postgres>) -> Result<()> {
    let tables = [
        query_file!("src/db/assets.sql"),
        query_file!("src/db/blobs.sql"),
        query_file!("src/db/blocks.sql"),
        query_file!("src/db/notes.sql"),
        query_file!("src/db/nullifiers.sql"),
        query_file!("src/db/validators.sql"),
        query_file!("src/db/validator_fundingstreams.sql"),
        query_file!("src/db/indexes/notes_height_idx.sql"),
        query_file!("src/db/indexes/notes_position_idx.sql"),
        query_file!("src/db/indexes/nullifiers_height_idx.sql"),
    ];

    let mut tx = db.begin().await?;
    for query in tables.into_iter() {
        query
            .execute(&mut tx)
            .await
            .context("could not initialize database")?;
    }
    tx.commit().await?;

    Ok(())
}
