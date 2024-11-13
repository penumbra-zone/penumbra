use std::fmt::Write as _;

use anyhow::{anyhow, Context};
use cnidarium::{StateDelta, Storage};
use penumbra_proto::{StateReadProto, StateWriteProto};

use super::APP_VERSION;

fn version_to_software_version(version: u64) -> &'static str {
    match version {
        1 => "v0.70.x",
        2 => "v0.73.x",
        3 => "v0.74.x",
        4 => "v0.75.x",
        5 => "v0.76.x",
        6 => "v0.77.x",
        7 => "v0.79.x",
        8 => "v0.80.x",
        _ => "unknown",
    }
}

#[derive(Debug, Clone, Copy)]
enum CheckContext {
    Running,
    Migration,
}

fn check_version(ctx: CheckContext, expected: u64, found: Option<u64>) -> anyhow::Result<()> {
    let found = match (expected, found) {
        (x, Some(y)) if x != y => y,
        _ => return Ok(()),
    };
    match ctx {
        CheckContext::Running => {
            let expected_name = version_to_software_version(expected);
            let found_name = version_to_software_version(expected);
            let mut error = String::new();
            error.push_str("app version mismatch:\n");
            write!(
                &mut error,
                "  expected {} (penumbra {})\n",
                expected, expected_name
            )?;
            write!(&mut error, "  found {} (penumbra {})\n", found, found_name)?;
            write!(
                &mut error,
                "make sure you're running penumbra {}",
                expected_name
            )?;
            Err(anyhow!(error))
        }
        CheckContext::Migration => {
            let expected_name = version_to_software_version(expected);
            let found_name = version_to_software_version(expected);
            let mut error = String::new();
            error.push_str("app version mismatch:\n");
            write!(
                &mut error,
                "  expected {} (penumbra {})\n",
                expected, expected_name
            )?;
            write!(&mut error, "  found {} (penumbra {})\n", found, found_name)?;
            write!(
                &mut error,
                "this migration should be run with penumbra {} instead",
                version_to_software_version(expected + 1)
            )?;
            Err(anyhow!(error))
        }
    }
}

fn state_key() -> Vec<u8> {
    b"penumbra_app_version_safeguard".to_vec()
}

async fn read_app_version_safeguard<S: StateReadProto>(s: &S) -> anyhow::Result<Option<u64>> {
    let out = s
        .nonverifiable_get_proto(&state_key())
        .await
        .context("while reading app version safeguard")?;
    Ok(out)
}

// Neither async nor a result are needed, but only right now, so I'm putting these here
// to reserve the right to change them later.
async fn write_app_version_safeguard<S: StateWriteProto>(s: &mut S, x: u64) -> anyhow::Result<()> {
    s.nonverifiable_put_proto(state_key(), x);
    Ok(())
}

/// Assert that the app version saved in the state is the correct one.
///
/// You should call this before starting a node.
///
/// This will succeed if no app version is saved, or if the app version saved matches
/// exactly.
///
/// This will also result in the current app version being stored, so that future
/// calls to this function will be checked against this state.
pub async fn assert_latest_app_version(s: Storage) -> anyhow::Result<()> {
    // If the storage is not initialized, avoid touching it at all,
    // to avoid complaints about it already being initialized before the first genesis.
    if s.latest_version() == u64::MAX {
        return Ok(());
    }
    let mut delta = StateDelta::new(s.latest_snapshot());
    let found = read_app_version_safeguard(&delta).await?;
    check_version(CheckContext::Running, APP_VERSION, found)?;
    write_app_version_safeguard(&mut delta, APP_VERSION).await?;
    s.commit(delta).await?;
    Ok(())
}

/// Migrate the app version to a given number.
///
/// This will check that the app version is currently the previous version, if set at all.
///
/// This is the only way to change the app version, and should be called during a migration
/// with breaking consensus logic.
pub async fn migrate_app_version<S: StateWriteProto>(s: &mut S, to: u64) -> anyhow::Result<()> {
    anyhow::ensure!(to > 1, "you can't migrate to the first penumbra version!");
    let found = read_app_version_safeguard(s).await?;
    check_version(CheckContext::Migration, to - 1, found)?;
    write_app_version_safeguard(s, to).await?;
    Ok(())
}
