use sqlx::PgPool;

use crate::database::read_only_db;

pub async fn missing_blocks(name: &'static str, db: PgPool) -> anyhow::Result<bool> {
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
    if let Some((current, next)) = res {
        println!("{}:", name);
        println!("missing blocks between heights {} and {}", current, next);
        return Ok(false);
    }
    Ok(true)
}

async fn missing_events(name: &'static str, db: PgPool) -> anyhow::Result<bool> {
    let mut dbtx = db.begin().await?;
    let res: Option<(i64, i64, i64)> = sqlx::query_as(
        "
      WITH event_heights AS (
        SELECT e.rowid, b.height,
               LEAD(e.rowid) OVER (PARTITION BY b.height ORDER BY e.rowid) AS next_rowid
        FROM events e
        JOIN blocks b ON e.block_id = b.rowid
      )
      SELECT height, rowid, next_rowid
      FROM event_heights
      WHERE next_rowid IS NOT NULL
      AND next_rowid - rowid <> 1
      ORDER BY height ASC, rowid ASC
      LIMIT 1;
    ",
    )
    .fetch_optional(dbtx.as_mut())
    .await?;
    if let Some((height, next, current)) = res {
        println!("{}:", name);
        println!(
            "missing events at height {}: missing between event #{} and #{}",
            height, current, next
        );
        return Ok(false);
    };
    dbtx.rollback().await?;
    Ok(true)
}

pub async fn integrity_check(src_database_url: &str) -> anyhow::Result<()> {
    let db = read_only_db(src_database_url).await?;
    let mut tasks = Vec::new();
    tasks.push(tokio::spawn({
        let db = db.clone();
        async move { missing_blocks("# 000 Missing Blocks", db).await }
    }));
    tasks.push(tokio::spawn({
        let db = db.clone();
        async move { missing_events("# 001 Missing Events", db).await }
    }));
    let mut failed = false;
    for task in tasks {
        failed |= !task.await??;
    }
    if failed {
        anyhow::bail!("integrity checks failed, check logs");
    }
    Ok(())
}
