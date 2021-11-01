use sqlx::postgres::{PgPoolOptions, PgQueryResult};
use sqlx::{query, Error};
use sqlx::{Pool, Postgres};
use std::env;

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

pub async fn db_bootstrap() -> Result<PgQueryResult, Error> {
    let mut conn = db_connection().await?;
    let bootstrap_sql = query(
        "
CREATE TABLE IF NOT EXISTS note_commitment_tree_anchors (
    id SERIAL PRIMARY KEY, 
    height bigint NOT NULL, 
    anchor bytea NOT NULL
)",
    )
    .execute(&conn)
    .await;

    bootstrap_sql
}
