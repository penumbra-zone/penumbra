use sqlx::PgPool;

use crate::database::read_only_db;

pub async fn missing_blocks(name: &'static str, db: PgPool) -> anyhow::Result<()> {
    let mut dbtx = db.begin().await?;
    let res: Option<(i64, i64)> = sqlx::query_as(
        "
      WITH height_gaps AS (
        SELECT
          height as current_height,
          LEAD(height) OVER (ORDER BY height) as next_height,
          LEAD(height) OVER (ORDER BY height) - height as gap
        FROM blocks
      )
      SELECT
        current_height,
        next_height
      FROM height_gaps
      WHERE gap > 1 AND next_height IS NOT NULL
      ORDER BY current_height ASC
      LIMIT 1;
    ",
    )
    .fetch_optional(dbtx.as_mut())
    .await?;
    dbtx.rollback().await?;
    match res {
        Some((current, next)) => {
            eprintln!("{}:", name);
            eprintln!("missing blocks between heights {} and {}", current, next);
            anyhow::bail!("{} check failed", name);
        }
        None => {}
    }
    Ok(())
}

pub async fn integrity_check(src_database_url: &str) -> anyhow::Result<()> {
    let db = read_only_db(src_database_url).await?;
    let mut tasks = Vec::new();
    tasks.push(tokio::spawn({
        let db = db.clone();
        async move { missing_blocks("# 000 Missing Blocks", db).await }
    }));
    let mut failed = false;
    for task in tasks {
        let res = task.await?;
        failed |= res.is_err();
    }
    if failed {
        anyhow::bail!("integrity checks failed, check logs");
    }
    Ok(())
}
