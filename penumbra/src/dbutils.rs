use std::env;

use sqlx::postgres::{PgPoolOptions, PgQueryResult};
use sqlx::{query, query_as, Error};
use sqlx::{Pool, Postgres};

use crate::dbschema::NoteCommitmentTreeAnchor;
use crate::dbschema::PenumbraNoteCommitmentTreeAnchor;

fn db_get_connection_string() -> String {
    let mut db_connection_string = "".to_string();

    match env::var("DATABASE_URL") {
        Ok(val) => db_connection_string = val,
        Err(_e) => {}
    }

    db_connection_string
}

pub async fn db_connection() -> Result<Pool<Postgres>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(db_get_connection_string().as_str())
        .await;

    pool
}

/// Bootstrap the database, creating tables if they do not exist.
pub async fn db_bootstrap(pool: Pool<Postgres>) -> Result<PgQueryResult, Error> {
    let bootstrap_sql_blocks = query(
        r#"
CREATE TABLE IF NOT EXISTS blocks (
    id SERIAL PRIMARY KEY, 
    height bigint NOT NULL, 
    anchor bytea NOT NULL
)
"#,
    )
    .execute(&pool)
    .await;

    let bootstrap_sql_notes = query(
        r#"
CREATE TABLE IF NOT EXISTS notes (
    id SERIAL PRIMARY KEY,
    note_commitment bytea NOT NULL,
    note_ciphertext bytea NOT NULL,
    ephemeral_key bytea NOT NULL,
    transaction_id bigint REFERENCES transactions (id),
)
"#,
    )
    .execute(&pool)
    .await;

    let bootstrap_sql_transactions = query(
        r#"
CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    transaction_id bytea NOT NULL,
    transaction bytea NOT NULL,
    block_id bigint REFERENCES blocks (id)
)
"#,
    )
    .execute(&pool)
    .await;

    // xx combine with bootstrap_sql_blocks, bootstrap_sql_notes
    bootstrap_sql_transactions
}

/// Hardcoded query for inserting one row
pub async fn db_insert(
    records: PenumbraNoteCommitmentTreeAnchor,
    pool: Pool<Postgres>,
) -> Result<u64, Error> {
    let record: NoteCommitmentTreeAnchor = records.into();
    let mut p = pool.acquire().await?;
    let id = query("INSERT INTO blocks (height, anchor) VALUES ($1, $2)")
        .bind(record.height)
        .bind(record.anchor)
        .execute(&mut p)
        .await?
        .rows_affected();

    Ok(id)
}

/// Hardcoded query for reading the first record
pub async fn db_read(pool: Pool<Postgres>) -> Result<Vec<PenumbraNoteCommitmentTreeAnchor>, Error> {
    let mut p = pool.acquire().await?;
    let rows = query_as::<_, NoteCommitmentTreeAnchor>(
        "SELECT id, height, anchor FROM blocks where id = 1;",
    )
    .fetch_one(&mut p)
    .await?;

    let res = rows.into();
    Ok(vec![res])
}
