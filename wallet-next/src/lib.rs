use sqlx::sqlite::SqlitePool;

// Stub code -- note that whatever code works with SQL has to be in the library,
// not in the binary, so that we can run `cargo sqlx prepare` against one crate.

pub async fn insert_table(pool: &SqlitePool) -> anyhow::Result<i64> {
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

pub async fn read_table(pool: &SqlitePool) -> anyhow::Result<String> {
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
