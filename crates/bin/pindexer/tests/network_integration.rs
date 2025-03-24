#![cfg(feature = "network-integration")]
//! Basic integration testing for pindexer.
//!
//! Performs queries against a local db to confirm core functionality,
//! such as all blocks are indexed. The setup and creation of the local db is not
//! managed by the Rust code. Use the nix env to ensure all relevant software
//! is installed. The `just smoke` target will run this logic under the proper settings.

use anyhow::Context;
use rstest::rstest;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use tokio::time::{sleep, Duration};

/// Hardcoded URL to the local PostgreSQL database for CometBFT ABCI events.
const COMETBFT_DATABASE_URL: &str =
    "postgresql://penumbra:penumbra@localhost:5432/penumbra_cometbft?sslmode=disable";

/// Hardcoded URL to the local PostgreSQL database for pindexer.
const PINDEXER_DATABASE_URL: &str =
    "postgresql://penumbra:penumbra@localhost:5432/penumbra_pindexer?sslmode=disable";

/// Reusable fn to connect to target postgres db, based on DATABASE_URL connection string.
async fn get_db_handle(database_url: &str) -> anyhow::Result<PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?)
}

/// Poll the CometBFT instance for its notion of current block height.
async fn get_current_height() -> anyhow::Result<u64> {
    let client = reqwest::Client::new();
    let cmt_url =
        std::env::var("PENUMBRA_NODE_CMT_URL").unwrap_or("http://localhost:26657".to_string());
    let r = client.get(format!("{}/status", cmt_url)).send().await?;

    assert_eq!(r.status(), reqwest::StatusCode::OK);

    let current_height: u64 = r
        .json::<serde_json::Value>()
        .await?
        .get_mut("result")
        .and_then(|v| v.get_mut("sync_info"))
        .and_then(|v| v.get_mut("latest_block_height"))
        .ok_or_else(|| anyhow::anyhow!("could not parse block height from cometbft json"))?
        .take()
        .as_str()
        .expect("could not find height in cometbft response")
        .parse()?;
    Ok(current_height)
}

/// Query the CometBFT PostgreSQL database, and report its highest known block.
async fn get_highest_indexed_block_from_cometbft_db() -> anyhow::Result<u64> {
    let conn = get_db_handle(COMETBFT_DATABASE_URL).await?;

    // Execute query and parse result as u64
    let row = sqlx::query("SELECT height FROM blocks ORDER BY height DESC LIMIT 1")
        .fetch_one(&conn)
        .await
        .context("failed to get results from cometbft db; is postgres indexing configured?")?;

    let result: i64 = row.get(0);
    let height: u64 = result.try_into()?;

    Ok(height)
}

/// Query the pindexer PostgreSQL database, and report its highest known block.
async fn get_highest_indexed_block_from_pindexer_db() -> anyhow::Result<u64> {
    let conn = get_db_handle(PINDEXER_DATABASE_URL).await?;

    // Execute query and parse result as u64
    let row = sqlx::query("SELECT height FROM block_details ORDER BY height DESC LIMIT 1")
        .fetch_one(&conn)
        .await
        .context("failed to get results from pindexer db; did pindexer crash?")?;

    let result: i64 = row.get(0);
    let height: u64 = result.try_into()?;

    Ok(height)
}

#[tokio::test]
/// Confirm that the devnet chain is creating new blocks. Sanity-check,
/// so that our assumptions about the events in the database are grounded.
async fn chain_is_progressing() -> anyhow::Result<()> {
    let height_1 = get_current_height().await?;
    assert!(height_1 > 0, "initial height is not greater than 0!");
    // The default block time is ~5s, so we'll wait a bit longer than that, to ensure the chain
    // has progressed by at least 1 block.
    sleep(Duration::from_secs(7)).await;

    let height_2 = get_current_height().await?;
    assert!(
        height_2 > height_1,
        "second height is not greater than initial height"
    );
    Ok(())
}

#[tokio::test]
/// Confirm that the highest height reported by the CometBFT RPC
/// matches what's reported by the database.
async fn cometbft_indexing_is_working() -> anyhow::Result<()> {
    let height_rpc = get_current_height().await?;
    let height_db: u64 = get_highest_indexed_block_from_cometbft_db().await?;
    assert!(
        vec![height_rpc, height_rpc - 1].contains(&height_db),
        "cometbft database is behind chain; is indexing broken?"
    );
    Ok(())
}

#[rstest]
/// Database queries for the CometBFT event database, checking for null values.
/// All of these queries should return zero records in a well-formed database.
#[case("SELECT COUNT(*) FROM block_events WHERE key IS NULL;")]
#[case("SELECT COUNT(*) FROM block_events WHERE value IS NULL;")]
#[case("SELECT COUNT(*) FROM block_events WHERE composite_key IS NULL;")]
#[case("SELECT COUNT(*) FROM event_attributes WHERE key IS NULL;")]
#[case("SELECT COUNT(*) FROM event_attributes WHERE value IS NULL;")]
#[case("SELECT COUNT(*) FROM event_attributes WHERE composite_key IS NULL;")]
#[tokio::test]
/// Assert structure of cometbft database. Even if we assume cometbft will
/// "do the right thing" with ABCI event data it receives from pd, we cannot
/// be sure that pd is in fact emitting properly constructed ABCI Events
/// unless we check what's in the db.
async fn cometbft_events_are_not_null(#[case] query: &str) -> anyhow::Result<()> {
    let conn = get_db_handle(COMETBFT_DATABASE_URL).await?;

    // Execute query and parse result as u64
    let row = sqlx::query(query)
        .fetch_one(&conn)
        .await
        .context("failed to get results from cometbft db")?;

    let result: i64 = row.get(0);
    let count: u64 = result.try_into()?;
    assert_eq!(
        count, 0,
        "found {} null keys in cometbft event db, via query: '{}'",
        count, query
    );
    Ok(())
}

#[tokio::test]
/// Confirm that the highest height reported by the pindexer db
/// matches what's in the cometbft db.
async fn pindexer_is_working() -> anyhow::Result<()> {
    let height_rpc = get_current_height().await?;
    let height_db: u64 = get_highest_indexed_block_from_pindexer_db().await?;
    // Check that pindexer's height is within tolerance. We allow lagging by ~4 blocks because
    // local devnets and integration test suites run with faster block times, ~1s vs the default
    // 5s, so being a bit behind is quite possible.
    const ALLOWANCE: u64 = 4;
    assert!(
        (height_rpc - ALLOWANCE..=height_rpc).contains(&height_db),
        "pindexer database is behind chain ({} vs {}); is indexing broken?",
        height_db,
        height_rpc
    );
    Ok(())
}
