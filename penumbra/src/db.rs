use sqlx::postgres::{PgPoolOptions, PgQueryResult, PgRow};
use sqlx::FromRow;
use sqlx::Row;
use sqlx::{query, query_as, Error};
use sqlx::{Pool, Postgres};
use std::env;

#[derive(Debug, sqlx::FromRow)]
pub struct NoteCommitmentTreeAnchor {
    pub id: i64,
    pub height: i64,
    pub anchor: Vec<u8>,
}

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

pub async fn db_bootstrap(pool: Pool<Postgres>) -> Result<PgQueryResult, Error> {
    let bootstrap_sql = query(
        r#"
CREATE TABLE IF NOT EXISTS note_commitment_tree_anchors (
    id SERIAL PRIMARY KEY, 
    height bigint NOT NULL, 
    anchor bytea NOT NULL
)"#,
    )
    .execute(&pool)
    .await;

    bootstrap_sql
}

pub async fn db_insert(
    record: NoteCommitmentTreeAnchor,
    pool: Pool<Postgres>,
) -> Result<u64, Error> {
    let mut p = pool.acquire().await?;
    let id = query("INSERT INTO note_commitment_tree_anchors (height, anchor) VALUES ($1, $2)")
        .bind(record.height)
        .bind(record.anchor)
        .execute(&mut p)
        .await?
        .rows_affected();

    Ok(id)
}
