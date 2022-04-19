use sqlx::{Pool, Sqlite};

pub struct Storage {
    pub(super) pool: Pool<Sqlite>,
}

impl Storage {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn migrate(self: &Storage) -> anyhow::Result<()> {
        sqlx::migrate!().run(&self.pool).await.map_err(Into::into)
    }

    pub async fn insert_table(self: &Storage) -> anyhow::Result<i64> {
        let mut conn = self.pool.acquire().await?;

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

    pub async fn read_table(self: &Storage) -> anyhow::Result<String> {
        let recs = sqlx::query!(
            r#"
SELECT id, value
FROM penumbra
ORDER BY id
LIMIT 1
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(recs[0].value.clone())
    }
}
