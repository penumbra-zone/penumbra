use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use std::env;

use penumbra_wallet_next::Storage;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    let storage = Storage::new(pool);

    storage.migrate().await?;

    let row = storage.insert_table().await?;
    let x = storage.read_table().await?;

    println!(
        "Hello, pwalletd! I got stuff from sqlite: row {} value {}",
        row, x
    );
    Ok(())
}
