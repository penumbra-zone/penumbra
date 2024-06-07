//! Contains functions related to the migration script of Testnet78.
use cnidarium::{StateDelta, Storage};
use std::time::Duration;
use tracing::instrument;

/// Run the full migration, given an export path and a start time for genesis.
///
/// Menu:
/// - Truncate various user-supplied `String` fields to a maximum length.
///   * Validator Definitions:
///    - `name` (140 characters)
///    - `website` (70 characters)
///    - `description` (280 characters)
///   * Governance Parameter Changes:
///    - `key` (64 characters)
///    - `value` (2048 characters)
///    - `component` (64 characters)
///   * Governance Proposals:
///    - `title` (80 characters)
///    - `description` (10,000 characters)
///   * Governance Proposal Withdrawals:
///    - `reason` (1024 characters)
///   * Governance IBC Client Freeze Proposals:
///    - `client_id` (128 characters)
///   * Signaling Proposals:
///    - `commit hash` (64 characters)
#[instrument]
pub async fn migrate(storage: Storage) -> anyhow::Result<Duration> {
    // Setup:
    let initial_state = storage.latest_snapshot();

    // We initialize a `StateDelta` and start by reaching into the JMT for all entries matching the
    // swap execution prefix. Then, we write each entry to the nv-storage.
    let mut delta = StateDelta::new(initial_state);
    let start_time = std::time::SystemTime::now();

    // TODO: adjust data lengths here

    let post_upgrade_root_hash = storage.commit_in_place(delta).await?;
    tracing::info!(?post_upgrade_root_hash, "post-migration root hash");

    Ok(start_time.elapsed().expect("start time not set"))
}
