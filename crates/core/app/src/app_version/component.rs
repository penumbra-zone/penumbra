use std::fmt::Write as _;

use anyhow::{anyhow, Context};
use cnidarium::{StateDelta, Storage};
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};

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
        9 => "v0.81.x",
        10 => "v1.4.x",
        11 => "v2.0.x",
        12 => "v2.1.x",
        _ => "unknown",
    }
}

#[derive(Debug, Clone, Copy)]
enum CheckContext {
    Running,
    Migration,
}

/// Check that the expected version matches the found version, if it set.
/// This will return an error if the versions do not match.
#[tracing::instrument]
fn check_version(ctx: CheckContext, expected: u64, found: Option<u64>) -> anyhow::Result<()> {
    tracing::debug!("running version check");
    let found = found.unwrap_or(expected);
    if found == expected {
        return Ok(());
    }

    match ctx {
        CheckContext::Running => {
            let expected_name = version_to_software_version(expected);
            let found_name = version_to_software_version(found);
            let mut error = String::new();
            error.push_str("app version mismatch:\n");
            write!(
                &mut error,
                "  expected {} (penumbra {})\n",
                expected, expected_name
            )?;
            write!(&mut error, "  found {} (penumbra {})\n", found, found_name)?;
            write!(&mut error, "Are you using the right node directory?\n")?;
            // For a greater difference, the wrong directory is probably being used.
            if found == expected - 1 {
                write!(&mut error, "Does a migration need to happen?\n")?;
                write!(
                    &mut error,
                    "If so, then run `pd migrate` with version {}",
                    expected_name
                )?;
            } else {
                write!(
                    &mut error,
                    "make sure you're running penumbra {}",
                    expected_name
                )?;
            }
            Err(anyhow!(error))
        }
        CheckContext::Migration => {
            let expected_name = version_to_software_version(expected);
            let found_name = version_to_software_version(found);
            let mut error = String::new();
            if found == APP_VERSION {
                write!(
                    &mut error,
                    "state already migrated to version {}",
                    APP_VERSION
                )?;
                anyhow::bail!(error);
            }
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

/// Read the app version safeguard from nonverifiable storage.
async fn read_app_version_safeguard<S: StateReadProto>(s: &S) -> anyhow::Result<Option<u64>> {
    let out = s
        .nonverifiable_get_proto(crate::app::state_key::app_version::safeguard().as_bytes())
        .await
        .context("while reading app version safeguard")?;
    Ok(out)
}

/// Write the app version safeguard to nonverifiable storage.
fn write_app_version_safeguard<S: StateWriteProto>(s: &mut S, x: u64) {
    s.nonverifiable_put_proto(
        crate::app::state_key::app_version::safeguard()
            .as_bytes()
            .to_vec(),
        x,
    )
}

/// Ensure that the app version safeguard is `APP_VERSION`, or update it if it is missing.
///
/// # Errors
/// This method errors if the app version safeguard is different than `APP_VERSION`.
///
/// # Usage
/// This should be called on startup. This method short-circuits if the database
/// is uninitialized (pregenesis).
///
/// # UIP:
/// More context is available in the UIP-6 document: https://uips.penumbra.zone/uip-6.html
pub async fn check_and_update_app_version(s: Storage) -> anyhow::Result<()> {
    // If the storage is not initialized, avoid touching it at all,
    // to avoid complaints about it already being initialized before the first genesis.
    if s.latest_version() == u64::MAX {
        return Ok(());
    }
    let mut delta = StateDelta::new(s.latest_snapshot());

    // If the safeguard is not set, set it to the current version.
    // Otherwise, ensure that it matches the current version.
    match read_app_version_safeguard(&delta).await? {
        None => {
            tracing::debug!(?APP_VERSION, "version safeguard not found, initializing");
            write_app_version_safeguard(&mut delta, APP_VERSION);
            s.commit_in_place(delta).await?;
        }
        Some(found) => check_version(CheckContext::Running, APP_VERSION, Some(found))?,
    }
    Ok(())
}

/// Migrate the app version to a given number.
///
/// This will check that the app version is currently the previous version, if set at all.
///
/// This is the recommended way to change the app version, and should be called during a migration
/// with breaking consensus logic.
pub async fn migrate_app_version<S: StateWriteProto>(s: &mut S, to: u64) -> anyhow::Result<()> {
    anyhow::ensure!(to > 1, "you can't migrate to the first penumbra version!");
    let found = read_app_version_safeguard(s).await?;
    check_version(CheckContext::Migration, to - 1, found)?;
    write_app_version_safeguard(s, to);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    /// Confirm there's a matching branch on the APP_VERSION to crate version lookup.
    /// It's possible to overlook that update when bumping the APP_VERSION, so this test
    /// ensures that if the APP_VERSION was changed, so was the match arm.
    fn ensure_app_version_is_current_in_checks() -> anyhow::Result<()> {
        let result = version_to_software_version(APP_VERSION);
        assert!(
            result != "unknown",
            "APP_VERSION lacks a corresponding software version"
        );
        Ok(())
    }
}
