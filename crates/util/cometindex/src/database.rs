use sqlx::{postgres::PgPoolOptions, PgPool};

/// Create a Database, with, for sanity, some read only settings.
///
/// These will be overrideable by a consumer who knows what they're doing,
/// but prevents basic mistakes.
/// c.f. https://github.com/launchbadge/sqlx/issues/481#issuecomment-727011811
pub async fn read_only_db(url: &str) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET SESSION CHARACTERISTICS AS TRANSACTION READ ONLY;")
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect(url)
        .await
        .map_err(Into::into)
}

pub async fn read_write_db(url: &str) -> anyhow::Result<PgPool> {
    PgPoolOptions::new().connect(url).await.map_err(Into::into)
}
