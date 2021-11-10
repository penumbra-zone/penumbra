use std::env;

use sqlx::postgres::{PgPoolOptions, PgQueryResult};
use sqlx::{query, query_as, Error};
use sqlx::{Pool, Postgres};

use crate::dbschema::NoteCommitmentTreeAnchor;
use crate::dbschema::PenumbraNoteCommitmentTreeAnchor;

use crate::state::PendingBlock;

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
    transaction bytea NOT NULL,
    block_id bigint REFERENCES blocks (id)
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
    ephemeral_key bytea NOT NULL,
    note_ciphertext bytea NOT NULL,
    transaction_id bigint REFERENCES transactions (id)
)
"#,
    )
    .execute(&pool)
    .await;

    // xx combine with bootstrap_sql_blocks, bootstrap_sql_notes
    bootstrap_sql_transactions
}

/// Hardcoded query for inserting one row into `blocks` given an active database transaction.
pub async fn db_insert_block(
    records: PenumbraNoteCommitmentTreeAnchor,
    dbtx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<u64, Error> {
    let record: NoteCommitmentTreeAnchor = records.into();
    let id = query("INSERT INTO blocks (height, anchor) VALUES ($1, $2)")
        .bind(record.height)
        .bind(record.anchor)
        .execute(dbtx)
        .await?
        .rows_affected();

    Ok(id)
}

/// Hardcoded query for inserting one row into `transactions` table given an active database transaction.
pub async fn db_insert_transaction(
    dbtx: &mut sqlx::Transaction<'_, Postgres>,
    transaction: Vec<u8>,
    block_id: u64,
) -> Result<u64, Error> {
    let id = query("INSERT INTO transactions (transaction, block_id) VALUES ($1, $2)")
        .bind(transaction)
        .bind(block_id as i64)
        .execute(dbtx)
        .await?
        .rows_affected();

    Ok(id)
}

/// Hardcoded query for inserting one row into `notes` table given an active database transaction.
pub async fn db_insert_note(
    dbtx: &mut sqlx::Transaction<'_, Postgres>,
    note_commitment: Vec<u8>,
    ephemeral_key: Vec<u8>,
    note_ciphertext: Vec<u8>,
    transaction_id: u64,
) -> Result<u64, Error> {
    // xx sqlx does not impl<'_> Encode<'_, Postgres> for u64
    // Weird because that is the type we're getting for the ID when we insert rows
    let id = query(
        "INSERT INTO notes (note_commitment, ephemeral_key, note_ciphertext, transaction_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(note_commitment)
    .bind(ephemeral_key)
    .bind(note_ciphertext)
    .bind(transaction_id as i64)
    .execute(dbtx)
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

/// Saves all pending state changes from a new block.
pub async fn db_commit_block(
    pool: Pool<Postgres>,
    block_state: PendingBlock,
    height: i64,
    anchor: [u8; 32],
) {
    let mut dbtx = pool.begin().await.expect("can start transaction");

    // Store the block in database table `blocks`.
    let block_pk = db_insert_block(
        PenumbraNoteCommitmentTreeAnchor::from(NoteCommitmentTreeAnchor {
            id: height as i32,
            height,
            anchor: anchor.to_vec(),
        }),
        &mut dbtx,
    )
    .await
    .unwrap();

    // Store each serialized transaction in database table `transactions` linked to the block primary key.
    for (transaction, note_fragments) in block_state.transactions {
        let transaction_pk = db_insert_transaction(&mut dbtx, transaction, block_pk)
            .await
            .expect("can get pk of new transaction");

        // Store each note fragment in database table `notes` linked to the transaction primary key.
        for fragment in note_fragments {
            println!("saving notes");
            db_insert_note(
                &mut dbtx,
                fragment.note_commitment.into(),
                fragment.ephemeral_key.0.to_vec(),
                fragment.note_ciphertext.to_vec(),
                transaction_pk,
            )
            .await
            .expect("can insert note into database");
        }
    }

    dbtx.commit()
        .await
        .expect("we can commit the state changes to the database")
}
