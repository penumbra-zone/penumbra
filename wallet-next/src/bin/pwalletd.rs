use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use std::env;

use penumbra_wallet_next::{insert_table, read_table};

#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    // TODO: weird chicken & egg problem w/ database existing or not
    sqlx::migrate!().run(&pool).await?;
    let row = insert_table(&pool).await?;
    let x = read_table(&pool).await?;

    println!(
        "Hello, pwalletd! I got stuff from sqlite: row {} value {}",
        row, x
    );
    Ok(())
}
