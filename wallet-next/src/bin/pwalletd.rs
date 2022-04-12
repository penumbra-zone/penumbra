use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    println!("Hello, pwalletd!");
    Ok(())
}
