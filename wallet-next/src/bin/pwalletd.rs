use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    // TODO: weird chicken & egg problem w/ database existing or not
    create_table(&pool).await?;
    let row = insert_table(&pool).await?;
    let x = read_table(&pool).await?;

    println!(
        "Hello, pwalletd! I got stuff from sqlite: row {} value {}",
        row, x
    );
    Ok(())
}

async fn create_table(pool: &SqlitePool) -> anyhow::Result<()> {
    //     let mut conn = pool.acquire().await?;

    //     // Create the table
    //     sqlx::query!(
    //         r#"
    // CREATE TABLE penumbra (id INTEGER PRIMARY KEY, value TEXT NOT NULL);
    //         "#
    //     )
    //     .execute(&mut conn)
    //     .await?;

    Ok(())
}

async fn insert_table(pool: &SqlitePool) -> anyhow::Result<i64> {
    let mut conn = pool.acquire().await?;

    // Insert the task, then obtain the ID of this row
    let id = sqlx::query!(
        r#"
INSERT INTO penumbra ( value )
VALUES ( ?1 )
        "#,
        "Hello, world"
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}

async fn read_table(pool: &SqlitePool) -> anyhow::Result<String> {
    let recs = sqlx::query!(
        r#"
SELECT id, value
FROM penumbra
ORDER BY id
LIMIT 1
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(recs[0].value.clone())
}
