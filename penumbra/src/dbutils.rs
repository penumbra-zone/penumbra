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

    let bootstrap_sql_transactions = query(
        r#"
CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    transaction bytea NOT NULL
)
"#,
    )
    .execute(&pool)
    .await;
    // xx Add:
    // transaction_id bytea NOT NULL,
    // block_id bigint REFERENCES blocks (id)
    // to this table?

    let bootstrap_sql_notes = query(
        r#"
CREATE TABLE IF NOT EXISTS notes (
    id SERIAL PRIMARY KEY,
    note_commitment bytea NOT NULL,
    transaction_id bigint REFERENCES transactions (id)
)
"#,
    )
    .execute(&pool)
    .await;
    // xx Add:
    //     ephemeral_key bytea NOT NULL,
    //     note_ciphertext bytea NOT NULL,

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

/// Hardcoded query for inserting one row into `transactions` table given an active transaction.
pub async fn db_insert_transaction(
    db_tx: &mut sqlx::Transaction<'static, Postgres>,
    transaction: Vec<u8>,
) -> Result<u64, Error> {
    let id = query("INSERT INTO transactions (transaction) VALUES ($1)")
        .bind(transaction)
        .execute(db_tx)
        .await?
        .rows_affected();

    Ok(id)
}

/// Hardcoded query for inserting one row into `notes` table given an active transaction.
pub async fn db_insert_note(
    db_tx: &mut sqlx::Transaction<'static, Postgres>,
    note_commitment: Vec<u8>,
    transaction_id: u64,
) -> Result<u64, Error> {
    // xx sqlx does not impl<'_> Encode<'_, Postgres> for u64
    // Weird because that is the type we're getting for the ID when we insert rows
    let id = query("INSERT INTO notes (note_commitment, transaction_id) VALUES ($1, $2)")
        .bind(note_commitment)
        .bind(transaction_id as u32)
        .execute(db_tx)
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
